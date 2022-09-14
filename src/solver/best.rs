use super::negamax;
use crate::{board, eval, stats::search};

/// Return all of the best moves if finished within the specified time with stats.
pub fn bestmoves_with_stats(
    node: &board::Board,
    timeout: std::time::Duration,
) -> (Result<(eval::Eval, Vec<(u8, u8)>), ()>, search::Stats) {
    let (send_timeout, recv_timeout) = std::sync::mpsc::channel();
    let (send_result, recv_result) = std::sync::mpsc::channel();

    let node_clone = node.clone();

    std::thread::spawn(move || {
        send_result
            .send(best(&node_clone, recv_timeout))
            .expect("Could not send result.");
    });

    match recv_result.recv_timeout(timeout) {
        Ok(result) => result, // this should always be Ok
        Err(_) => {
            send_timeout.send(()).expect("Could not send timeout.");
            recv_result
                .recv()
                .expect("Could not wait until thread terminated.")
        }
    }
}

/// Return all of the best moves if finished within the specified time.
pub fn bestmoves(node: &board::Board, timeout: std::time::Duration) -> Result<Vec<(u8, u8)>, ()> {
    // If there is only one possible move we immediately return this move.
    let moves = node.moves();
    if moves.len() == 1 {
        return Ok(moves);
    }

    let (_eval, moves) = bestmoves_with_stats(node, timeout).0?;
    Ok(moves)
}

fn best(
    node: &board::Board,
    recv_timeout: std::sync::mpsc::Receiver<()>,
) -> (Result<(eval::Eval, Vec<(u8, u8)>), ()>, search::Stats) {
    let now = std::time::Instant::now();

    let mut negamax = negamax::Negamax::new(recv_timeout, node.movecount(), node.onturn());
    let mut bestmoves: Vec<(u8, u8)> = Vec::new();
    let mut max = eval::Eval::MIN;

    let mut moves = node.moves();
    moves.sort_by(|(_s1, c1), (_s2, _c2)| match node.isfull(*c1) {
        true => std::cmp::Ordering::Greater,
        false => std::cmp::Ordering::Less,
    });

    // TODO IMPORTANT add a form of the MTD(f) search
    for &(square, cell) in &moves {
        let mut child = node.clone();
        child.play(square, cell);

        let alpha = eval::Eval::MIN;
        let beta = eval::Eval::MAX;

        // TODO reuse improved alpha (beta does not change here)
        let value = negamax::eval(&child, alpha, beta, &mut negamax);

        if value.is_err() {
            negamax.stats.table = negamax.table.stats();
            negamax.stats.time = now.elapsed();

            return (Err(()), negamax.stats);
        }

        let value = -value.unwrap();
        if value > max {
            max = value;
            bestmoves.clear();
            bestmoves.push((square, cell));
        } else if value == max {
            bestmoves.push((square, cell));
        }
    }

    negamax.stats.table = negamax.table.stats();
    negamax.stats.time = now.elapsed();

    (Ok((max, bestmoves)), negamax.stats)
}
