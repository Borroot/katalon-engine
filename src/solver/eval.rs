use super::negamax;
use crate::{board, eval, stats::search};

/// Evaluate all the possible moves from the current board position.
// TODO pub fn eval_all()

/// Evaluate the current board position and provide stats.
pub fn eval_with_stats(
    node: &board::Board,
    timeout: std::time::Duration,
) -> (Result<eval::Eval, ()>, search::Stats) {
    let (send_timeout, recv_timeout) = std::sync::mpsc::channel();
    let (send_result, recv_result) = std::sync::mpsc::channel();

    let node_clone = node.clone();

    std::thread::spawn(move || {
        send_result
            .send(eval_plain(&node_clone, recv_timeout))
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

/// Evaluate the current board position.
pub fn eval(
    node: &board::Board,
    timeout: std::time::Duration,
) -> Result<eval::Eval, ()> {
    eval_with_stats(node, timeout).0
}

/// Evaluate the current board position just with negamax.
#[allow(dead_code)]
fn eval_plain(
    node: &board::Board,
    recv_timeout: std::sync::mpsc::Receiver<()>,
) -> (Result<eval::Eval, ()>, search::Stats) {
    let mut negamax = negamax::Negamax::new(recv_timeout, node.movecount(), node.onturn());

    negamax.stats.stopwatch_start();

    let alpha = eval::Eval::MIN;
    let beta = eval::Eval::MAX;
    let result = negamax::eval(&node, alpha, beta, &mut negamax);

    negamax.stats.stopwatch_stop();
    negamax.stats.add_table(negamax.table.stats());

    match result {
        Ok(result) => (Ok(result), negamax.stats),
        Err(_) => {
            negamax.stats.timeout();
            (Err(()), negamax.stats)
        }
    }
}

/// Evaluate the current board position using MTD(f).
#[allow(dead_code)]
fn eval_mtdf(
    node: &board::Board,
    recv_timeout: std::sync::mpsc::Receiver<()>,
) -> (Result<eval::Eval, ()>, search::Stats) {
    let mut negamax = negamax::Negamax::new(recv_timeout, node.movecount(), node.onturn());

    let mut max = eval::Eval::MAX.raw();
    let mut min = eval::Eval::MIN.raw();
    let mut guess = 0;

    negamax.stats.stopwatch_start();

    while min < max {
        negamax.stats.nullwindows += 1;
        let beta = std::cmp::max(guess, min + 1);
        let alpha = eval::Eval::new(beta - 1);

        let result = negamax::eval(&node, alpha, eval::Eval::new(beta), &mut negamax);
        if result.is_err() {
            negamax.stats.stopwatch_stop();
            negamax.stats.add_table(negamax.table.stats());
            negamax.stats.timeout();
            return (Err(()), negamax.stats);
        }
        guess = result.unwrap().raw();

        if guess < beta {
            max = guess;
        } else {
            min = guess;
        }
    }

    negamax.stats.stopwatch_stop();
    negamax.stats.add_table(negamax.table.stats());

    (Ok(eval::Eval::new(guess)), negamax.stats)
}

/// Evaluate the current board position using a binary search with null windows.
#[allow(dead_code)]
fn eval_divide(
    node: &board::Board,
    recv_timeout: std::sync::mpsc::Receiver<()>,
) -> (Result<eval::Eval, ()>, search::Stats) {
    let mut negamax = negamax::Negamax::new(recv_timeout, node.movecount(), node.onturn());

    let mut max = eval::Eval::MAX.raw();
    let mut min = eval::Eval::MIN.raw();
    let mut mid;

    negamax.stats.stopwatch_start();

    loop {
        negamax.stats.nullwindows += 1;
        mid = (min + max) / 2;

        let alpha = eval::Eval::new(mid - 1);
        let beta = eval::Eval::new(mid + 1);

        let result = negamax::eval(&node, alpha, beta, &mut negamax);
        if result.is_err() {
            negamax.stats.stopwatch_stop();
            negamax.stats.add_table(negamax.table.stats());
            negamax.stats.timeout();
            return (Err(()), negamax.stats);
        }
        let result = result.unwrap().raw();

        if mid == result {
            break;
        } else if result < mid {
            max = result;
        } else {
            min = result;
        }
    }

    negamax.stats.stopwatch_stop();
    negamax.stats.add_table(negamax.table.stats());

    (Ok(eval::Eval::new(mid)), negamax.stats)
}
