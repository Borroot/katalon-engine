use super::table;
use crate::{board, eval, player};

// TODO add a function which evaluates all the moves

/// The statistics of a given search performed by the solver.
pub struct Stats {
    /// The amount of time the solver took in total.
    pub time: std::time::Duration,
    /// The number of states that were evaluated.
    pub visited: usize,
    /// The reached search depth in iterative deepening.
    pub depth: usize,
    /// If the timeout was reached.
    pub timeout: bool,
    /// Table information.
    pub table: table::Stats,
    pub correct: bool, // TODO remove me
}

impl Stats {
    pub fn new() -> Self {
        Self {
            time: std::time::Duration::ZERO,
            visited: 0,
            depth: 0,
            timeout: false,
            table: table::Stats::new(),
            correct: true,
        }
    }
}

impl std::fmt::Display for Stats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            concat!(
                "total search time: {}ms\n",
                "states visited: {}\n",
                "depth reached: {}\n",
                "timeout: {}\n",
                "correct: {}\n",
                "{}\n",
            ),
            self.time.as_millis(),
            self.visited,
            self.depth,
            self.timeout,
            self.correct,
            self.table,
        )
    }
}

/// Various variables needed during the minmax search.
struct Minmax {
    recv_timeout: std::sync::mpsc::Receiver<()>,
    rootcount: i16,
    rootplayer: player::Players,
    table: table::Table,
    stats: Stats,
}

/// Return all of the best moves and the evaluation.
pub fn bestmoves(
    node: &board::Board,
    recv_timeout: std::sync::mpsc::Receiver<()>,
) -> (Result<(eval::Eval, Vec<(u8, u8)>), ()>, Stats) {
    let now = std::time::Instant::now();

    let mut minmax = Minmax {
        recv_timeout,
        rootcount: node.movecount(),
        rootplayer: node.onturn(),
        table: table::Table::from_gb(1.0),
        stats: Stats::new(),
    };

    let mut bestmoves: Vec<(u8, u8)> = Vec::new();
    let mut max = eval::Eval::MIN;

    let moves = node.moves();
    // TODO sort the moves

    // TODO add parallelization
    // TODO add iterative deepening and null window search
    for (square, cell) in moves {
        let mut child = node.clone();
        child.play(square, cell);

        // TODO reuse improved alpha (beta does not change here)
        let value = negamax(&child, eval::Eval::MIN, eval::Eval::MAX, &mut minmax);
        if value.is_err() {
            minmax.stats.table = minmax.table.stats();
            minmax.stats.time = now.elapsed();

            return (Err(()), minmax.stats);
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

    minmax.stats.table = minmax.table.stats();
    minmax.stats.time = now.elapsed();

    (Ok((max, bestmoves)), minmax.stats)
}

/// Evaluate the board from the perspective of the player onturn.
fn evaluation(
    result: board::Result,
    onturn: player::Players,
    rootplayer: player::Players,
    distance: i16,
) -> eval::Eval {
    // First evaluate from the perspective of the rootplayer.
    let result = match result.player() {
        Some(player) if player == rootplayer => eval::Result::Win,
        Some(_) => eval::Result::Loss,
        None => eval::Result::Draw,
    };

    // Second convert to the perspective of the player onturn.
    if rootplayer != onturn {
        -eval::Eval::from(result, distance)
    } else {
        eval::Eval::from(result, distance)
    }
}

// TODO remove me
#[rustfmt::skip]
fn negamax(
    node: &board::Board,
    mut alpha: eval::Eval,
    mut beta: eval::Eval,
    minmax: &mut Minmax,
) -> Result<eval::Eval, ()> {
    debug_assert!(alpha < beta);

    // Check if the timeout is reached and we should interrupt the search.
    let recv = minmax.recv_timeout.try_recv();
    if recv.is_ok() || recv == Err(std::sync::mpsc::TryRecvError::Disconnected) {
        minmax.stats.timeout = true;
        return Err(());
    }

    minmax.stats.visited += 1;
    let alpha_original = alpha; // TODO remove me

    // Check if we have already seen this node before.
    // TODO also check symmetries if depth is low.
    if let Some(mut entry) = minmax.table.get(node.key()) {
        if let (eval::Result::Draw, distance) = entry.value.human() {
            if (distance > 0 && node.onturn() != minmax.rootplayer) ||
               (distance < 0 && node.onturn() == minmax.rootplayer)
            {
                entry.value = -entry.value;
            }
        }

        let table_value = entry.value.absolute(minmax.rootcount, node.movecount());

        match entry.flag {
            table::Flag::EXACT => return Ok(table_value),
            table::Flag::LOWERBOUND => alpha = std::cmp::max(alpha, table_value),
            table::Flag::UPPERBOUND => beta = std::cmp::min(beta, table_value),
        }
        if alpha >= beta {
            return Ok(table_value);
        }
    }

    // Compute the value of the leaf node
    if let Some(result) = node.isover() {
        return Ok(evaluation(
            result,
            node.onturn(),
            minmax.rootplayer,
            node.movecount() - minmax.rootcount,
        ));
    }

    let moves = node.moves();
    // TODO sort the moves based on iterative deepening results
    // TODO add a table which saves the order of children nodes
    // TODO killer heuristic

    let mut best = eval::Eval::MIN;

    for (square, cell) in moves {
        let mut child = node.clone();
        child.play(square, cell);

        best = std::cmp::max(
            best,
            -negamax(&child, -beta, -alpha, minmax)?,
        );

        alpha = std::cmp::max(alpha, best);
        if alpha >= beta {
            break;
        }
    }

    if let Some(mut entry) = minmax.table.get(node.key())  {
        if let (eval::Result::Draw, distance) = entry.value.human() {
            if (distance > 0 && node.onturn() != minmax.rootplayer) ||
               (distance < 0 && node.onturn() == minmax.rootplayer)
            {
                entry.value = -entry.value;
            }
        }

        let table_value = entry.value.absolute(minmax.rootcount, node.movecount());
        let table_value_original = entry.value.absolute(minmax.rootcount, entry.movecount);

        if alpha_original < best && best < beta && alpha_original < table_value && table_value < beta {
            match entry.flag {
                table::Flag::EXACT if best != table_value => {
                    minmax.stats.correct = false;
                    print!("\nboard: \n{}", node);
                    println!("invalid exact");
                    println!("actual value: {}, != table_value: {}", best, table_value);
                    println!("entry.value: {}, table_value_original {}", entry.value, table_value_original);
                    println!("alpha old: {}, alpha: {}, beta: {}", alpha_original, alpha, beta);
                    println!("entry.alpha: {}, entry.beta: {}", entry.alpha, entry.beta);
                    println!("movecount: {}, entry.movecount: {}, rootcount: {}", node.movecount(), entry.movecount, minmax.rootcount);
                    println!("takestreak: {}, square: {}, onturn: {}", node.takestreak(), node.square().unwrap(), node.onturn());
                }
                table::Flag::LOWERBOUND if best < table_value => {
                    minmax.stats.correct = false;
                    print!("\nboard: \n{}", node);
                    println!("invalid lowerbound");
                    println!("actual value: {}, < table_value: {}", best, table_value);
                    println!("entry.value: {}, table_value_original {}", entry.value, table_value_original);
                    println!("alpha old: {}, alpha: {}, beta: {}", alpha_original, alpha, beta);
                    println!("entry.alpha: {}, entry.beta: {}", entry.alpha, entry.beta);
                    println!("movecount: {}, entry.movecount: {}, rootcount: {}", node.movecount(), entry.movecount, minmax.rootcount);
                    println!("takestreak: {}, square: {}, onturn: {}", node.takestreak(), node.square().unwrap(), node.onturn());
                }
                table::Flag::UPPERBOUND if best > table_value => {
                    minmax.stats.correct = false;
                    print!("\nboard: \n{}", node);
                    println!("invalid upperbound");
                    println!("actual value: {}, > table_value: {}", best, table_value);
                    println!("entry.value: {}, table_value_original {}", entry.value, table_value_original);
                    println!("alpha old: {}, alpha: {}, beta: {}", alpha_original, alpha, beta);
                    println!("entry.alpha: {}, entry.beta: {}", entry.alpha, entry.beta);
                    println!("movecount: {}, entry.movecount: {}, rootcount: {}", node.movecount(), entry.movecount, minmax.rootcount);
                    println!("takestreak: {}, square: {}, onturn: {}", node.takestreak(), node.square().unwrap(), node.onturn());
                }
                _ => (),
            }
        }
    }

    let flag = {
        if best <= alpha_original {
            table::Flag::UPPERBOUND
        } else if best >= beta {
            table::Flag::LOWERBOUND
        } else {
            table::Flag::EXACT
        }
    };
    let table_best = best.relative(minmax.rootcount, node.movecount());
    minmax.table.put(
        node.key(),
        table_best,
        flag,
        0,
        alpha_original,
        beta,
        node.movecount(),
    ); // TODO add bestmove

    Ok(best)
}
