use crate::{board, eval, player, table};
use rand::Rng;

// TODO refactor to a directory including: solver.rs, table.rs, eval.rs

/// A player directed by the minmax algorithm.
pub struct Solver;

impl player::Player for Solver {
    fn play(&self, node: &board::Board) -> (u8, u8) {
        let (_, bestmoves) = bestmoves(node, std::time::Duration::MAX).unwrap();
        let mut rng = rand::thread_rng();
        bestmoves[rng.gen_range(0..bestmoves.len()) as usize]
    }
}

/// Return all of the best moves if finished within the specified time.
pub fn bestmoves(
    node: &board::Board,
    timeout: std::time::Duration,
) -> Result<(eval::Eval, Vec<(u8, u8)>), ()> {
    let (send_timeout, recv_timeout) = std::sync::mpsc::channel();
    let (send_result, recv_result) = std::sync::mpsc::channel();

    let node_clone = node.clone();

    std::thread::spawn(move || {
        send_result
            .send(evaluate_start(&node_clone, recv_timeout))
            .expect("Could not send result.");
    });

    match recv_result.recv_timeout(timeout) {
        Ok(result) => result, // this should always be Ok
        Err(_) => {
            send_timeout.send(()).expect("Could not send timeout.");
            recv_result.recv().expect("Could not wait until thread terminated.")
        }
    }
}

/// Return all of the best moves and the pure evaluation.
fn evaluate_start(
    node: &board::Board,
    recv_timeout: std::sync::mpsc::Receiver<()>,
) -> Result<(eval::Eval, Vec<(u8, u8)>), ()> {
    let mut bestmoves: Vec<(u8, u8)> = Vec::new();
    let mut max = eval::Eval::MIN;

    let rootcount = node.movecount();
    let moves = moves(&node);

    let tablesize = 19_999_999; // TODO optimise table size
    let mut table = table::Table::<eval::Eval>::new(tablesize);

    // TODO add parallelization
    // TODO add iterative deepening and null window search
    for (square, cell) in moves {
        let mut child = node.clone();
        child.play(square, cell);

        let value = negamax(
            &child,
            eval::Eval::MIN,
            eval::Eval::MAX,
            rootcount,
            &mut table,
            &recv_timeout,
        )?
        .rev();

        if value > max {
            max = value;
            bestmoves.clear();
            bestmoves.push((square, cell));
        } else if value == max {
            bestmoves.push((square, cell));
        }
    }

    // TODO remove me
    // let num = (0..tablesize).filter(|&i| table.table[i].1 != None).count();
    // println!(
    //     "table sparsity {} ({:.2})",
    //     num,
    //     num as f64 / tablesize as f64
    // );

    // TODO return the evaluation of all the moves
    Ok((max, bestmoves))
}

/// Evaluate the board from the perspective of the player onturn.
fn evaluate_end(result: board::Result, onturn: player::Players, movecount: u8) -> eval::Eval {
    let result = match result.player() {
        Some(player) if player == onturn => eval::Result::Win,
        Some(_) => eval::Result::Loss,
        None => eval::Result::Draw,
    };
    eval::Eval::new(result, movecount)
}

/// Return all the moves that can be made from the given position.
fn moves(node: &board::Board) -> Vec<(u8, u8)> {
    if node.isfirst() {
        return vec![(0, 0), (0, 1), (0, 2), (0, 4), (2, 0), (2, 2)];
    } else {
        return vec![0, 1, 2, 3, 4]
            .into_iter()
            .map(|cell| (node.square().unwrap(), cell))
            .filter(|&(square, cell)| node.canplay(square, cell))
            .collect();
    }
}

fn negamax(
    node: &board::Board,
    mut alpha: eval::Eval,
    beta: eval::Eval,
    rootcount: u8,
    table: &mut table::Table<eval::Eval>,
    recv_timeout: &std::sync::mpsc::Receiver<()>,
) -> Result<eval::Eval, ()> {
    // Check if the timeout is reached and we should interrupt the search.
    let recv = recv_timeout.try_recv();
    if recv.is_ok() || recv == Err(std::sync::mpsc::TryRecvError::Disconnected) {
        return Err(());
    }

    // Compute the value of the leaf node
    if let Some(result) = node.isover() {
        return Ok(evaluate_end(
            result,
            node.onturn(),
            node.movecount() - rootcount,
        ));
    }

    // Check if we have already seen this node before.
    // TODO also check symmetries if depth is low.
    if let Some(eval) = table.get(node.key()) {
        return Ok(eval);
    }

    let moves = moves(&node);
    // TODO sort the moves

    let mut value = eval::Eval::MIN;

    for (square, cell) in moves {
        let mut child = node.clone();
        child.play(square, cell);

        value = std::cmp::max(
            value,
            negamax(
                &child,
                beta.rev(),
                alpha.rev(),
                rootcount,
                table,
                recv_timeout,
            )?
            .rev(),
        );

        table.put(node.key(), value);

        alpha = std::cmp::max(alpha, value);
        if alpha >= beta {
            break;
        }
    }
    return Ok(value);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::player::Player;

    /// Test if minmax detects the winning play.
    #[test]
    fn win_one_option() {
        // As player1, depth 1
        let board1 = board::Board::load("202123242").unwrap();
        assert_eq!(Solver.play(&board1), (2, 2));
        assert_eq!(bestmoves(&board1).0, eval::Eval::new(eval::Result::Win, 1));

        // As player2, depth 1
        let board2 = board::Board::load("0020103040").unwrap();
        assert_eq!(Solver.play(&board2), (0, 0));
        assert_eq!(bestmoves(&board2).0, eval::Eval::new(eval::Result::Win, 1));

        // As player1, depth 2
        let board3 = board::Board::load("01234321042244114110033").unwrap();
        assert_eq!(bestmoves(&board3).0, eval::Eval::new(eval::Result::Win, 2));
    }

    #[test]
    fn win_two_options() {
        // As player2, depth 3
        let board1 = board::Board::load("2200103024131211424323").unwrap();
        assert_eq!(Solver.play(&board1), (3, 3));
        assert_eq!(bestmoves(&board1).0, eval::Eval::new(eval::Result::Win, 3));
    }

    /// Test if minmax detects its gonna lose.
    #[test]
    fn loss_one_option() {
        // As player1, depth 2
        let board1 = board::Board::load("22001030241312114243233").unwrap();
        assert_eq!(Solver.play(&board1), (3, 4));
        assert_eq!(bestmoves(&board1).0, eval::Eval::new(eval::Result::Loss, 2));
    }

    /// Test if minmax detects its gonna be a draw.
    #[test]
    fn draw() {
        // The game is such that it is one move away from reaching the
        // takestreak, and it always has to take a stone, thus no matter which
        // move is made, it will be a draw.
        let start = String::from("20033102212432011410302234201");
        let cycle = "21103".repeat(6);
        let cycle = cycle
            .get(..board::Board::TAKESTREAK_LIMIT as usize - 3)
            .unwrap();

        let board = board::Board::load(&(start + cycle)).unwrap();
        assert_eq!(bestmoves(&board).0, eval::Eval::new(eval::Result::Draw, 1));
    }
}
