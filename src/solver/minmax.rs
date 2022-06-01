use super::{stats, table};
use crate::{board, eval, player};

// TODO add a function which evaluates all the moves

/// Return all of the best moves and the evaluation.
pub fn bestmoves(
    node: &board::Board,
    recv_timeout: std::sync::mpsc::Receiver<()>,
) -> (Result<(eval::Eval, Vec<(u8, u8)>), ()>, stats::Stats) {
    let mut stats = stats::Stats::new();

    let mut bestmoves: Vec<(u8, u8)> = Vec::new();
    let mut max = eval::Eval::MIN;

    let rootcount = node.movecount();
    let moves = legal_moves(&node);
    // TODO sort the moves

    let tablesize = 19_999_999; // TODO optimise table size
    let mut table = table::Table::<eval::Eval>::new(tablesize);

    // TODO add parallelization
    // TODO add iterative deepening and null window search
    for (square, cell) in moves {
        let mut child = node.clone();
        child.play(square, cell);

        // TODO reuse improved alpha and beta
        let value = negamax(
            &child,
            eval::Eval::MIN,
            eval::Eval::MAX,
            rootcount,
            &mut table,
            &recv_timeout,
            &mut stats,
        );
        if value.is_err() {
            stats.add_table(&table);
            stats.end();
            return (Err(()), stats);
        }

        let value = value.unwrap().rev();
        if value > max {
            max = value;
            bestmoves.clear();
            bestmoves.push((square, cell));
        } else if value == max {
            bestmoves.push((square, cell));
        }
    }

    stats.add_table(&table);
    stats.end();
    return (Ok((max, bestmoves)), stats);
}

/// Evaluate the board from the perspective of the player onturn.
fn evaluation(result: board::Result, onturn: player::Players, movecount: u8) -> eval::Eval {
    let result = match result.player() {
        Some(player) if player == onturn => eval::Result::Win,
        Some(_) => eval::Result::Loss,
        None => eval::Result::Draw,
    };
    eval::Eval::new(result, movecount)
}

/// Return all the moves that can be made from the given position.
fn legal_moves(node: &board::Board) -> Vec<(u8, u8)> {
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
    stats: &mut stats::Stats,
) -> Result<eval::Eval, ()> {
    // Check if the timeout is reached and we should interrupt the search.
    let recv = recv_timeout.try_recv();
    if recv.is_ok() || recv == Err(std::sync::mpsc::TryRecvError::Disconnected) {
        return Err(());
    }

    stats.add_visited();

    // Compute the value of the leaf node
    if let Some(result) = node.isover() {
        return Ok(evaluation(
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

    let moves = legal_moves(&node);
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
                stats,
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
