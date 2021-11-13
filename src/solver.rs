use crate::{board, eval, player, table};
use rand::prelude::*;
use std::cmp;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

/// A player directed by the minmax algorithm.
pub struct Solver;

impl player::Player for Solver {
    fn play(&self, node: &board::Board) -> (u8, u8) {
        let (_, bestmoves) = bestmoves(node);
        let mut rng = rand::thread_rng();
        bestmoves[rng.gen_range(0..bestmoves.len()) as usize]
    }
}

/// Return all of the best moves and the pure evaluation.
pub fn bestmoves(node: &board::Board) -> (eval::Eval, Vec<(u8, u8)>) {
    let mut bestmoves: Vec<(u8, u8)> = Vec::new();
    let mut max = eval::Eval::MIN;

    let rootcount = node.movecount();
    let moves = moves(&node);

    let mut table = table::Table::<eval::Eval>::new(100003);

    for (square, cell) in moves {
        let mut child = node.clone();
        child.play(square, cell);

        let value = negamax(
            &child,
            eval::Eval::MIN,
            eval::Eval::MAX,
            rootcount,
            &mut table,
        )
        .rev();

        if value > max {
            max = value;
            bestmoves.clear();
            bestmoves.push((square, cell));
        } else if value == max {
            bestmoves.push((square, cell));
        }
    }
    // TODO return the evaluation of all the moves
    (max, bestmoves)
}

/// Return all of the best moves if finished within the specified time.
pub fn bestmoves_timeout(
    node: &board::Board,
    timeout: Duration,
) -> Result<(eval::Eval, Vec<(u8, u8)>), mpsc::RecvTimeoutError> {
    let (sender, receiver) = mpsc::channel();
    let node_clone = node.clone();

    thread::spawn(move || {
        let _ = sender.send(bestmoves(&node_clone));
    });
    receiver.recv_timeout(timeout)
}

/// Evaluate the board from the perspective of the player onturn.
fn evaluate(result: board::Result, onturn: player::Players, movecount: u8) -> eval::Eval {
    let result = match result.player() {
        Some(player) => {
            if player == onturn {
                eval::Result::Win
            } else {
                eval::Result::Loss
            }
        }
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
) -> eval::Eval {
    // Compute the value of the leaf node
    let result = node.isover();
    if result != None {
        return evaluate(result.unwrap(), node.onturn(), node.movecount() - rootcount);
    }

    // Check if we have already seen this node before.
    if let Some(eval) = table.get(node.key()) {
        return eval;
    }

    // Generate and sort the children
    let moves = moves(&node);
    // TODO sort

    let mut value = eval::Eval::MIN;

    for (square, cell) in moves {
        let mut child = node.clone();
        child.play(square, cell);

        value = cmp::max(
            value,
            negamax(&child, beta.rev(), alpha.rev(), rootcount, table).rev(),
        );

        // Save the evaluation in the table.
        table.put(node.key(), value);

        // Perform alpha beta pruning.
        alpha = cmp::max(alpha, value);
        if alpha >= beta {
            break;
        }
    }
    return value;
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
