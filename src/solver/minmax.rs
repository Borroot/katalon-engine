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
    rootcount: u8,
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
        table: table::Table::from_gb(0.01),
        stats: Stats::new(),
    };

    let mut bestmoves: Vec<(u8, u8)> = Vec::new();
    let mut max = eval::Eval::MIN;

    let moves = legal_moves(&node);
    // TODO sort the moves

    // TODO add parallelization
    // TODO add iterative deepening and null window search
    for (square, cell) in moves {
        let mut child = node.clone();
        child.play(square, cell);

        // TODO reuse improved alpha and beta
        let value = negamax(&child, eval::Eval::MIN, eval::Eval::MAX, &mut minmax);
        if value.is_err() {
            minmax.stats.table = minmax.table.stats();
            minmax.stats.time = now.elapsed();

            return (Err(()), minmax.stats);
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

    minmax.stats.table = minmax.table.stats();
    minmax.stats.time = now.elapsed();

    (Ok((max, bestmoves)), minmax.stats)
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
        vec![(0, 0), (0, 1), (0, 2), (0, 4), (2, 0), (2, 2)]
    } else {
        (0..=4)
            .map(|cell| (node.square().unwrap(), cell))
            .filter(|&(square, cell)| node.canplay(square, cell))
            .collect()
    }
}

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

    let eval_relative = |value: &eval::Eval, count: u8| -> eval::Eval {
        eval::Eval::new(
            value.result.clone(),
            match value.result {
                eval::Result::Win | eval::Result::Loss => value.distance - count,
                eval::Result::Draw => value.distance,
            }
        )
    };

    let eval_absolute = |value: &eval::Eval, count: u8| -> eval::Eval {
        eval::Eval::new(
            value.result.clone(),
            match value.result {
                eval::Result::Win | eval::Result::Loss => value.distance + count,
                eval::Result::Draw => value.distance,
            }
        )
    };

    // Check if we have already seen this node before.
    // TODO also check symmetries if depth is low.
    //if let Some(entry) = minmax.table.get(node.key()) {
    //    let table_value = eval_absolute(&entry.value, node.movecount() - minmax.rootcount);
    //    match entry.flag {
    //        table::Flag::EXACT => return Ok(table_value),
    //        table::Flag::LOWERBOUND => alpha = std::cmp::max(alpha, table_value),
    //        table::Flag::UPPERBOUND => beta = std::cmp::min(beta, table_value),
    //    }
    //    if alpha >= beta {
    //        return Ok(table_value);
    //    }
    //}

    // Compute the value of the leaf node
    if let Some(result) = node.isover() {
        let e = Ok(evaluation(
            result,
            node.onturn(),
            node.movecount() - minmax.rootcount,
        ));
        //println!("\nboard: \n{}", node);
        //println!("evaluation: {}", e.unwrap());
        //println!("alpha old: {}, alpha: {}, beta: {}", alpha_original, alpha, beta);
        //println!("takestreak: {}, square: {}, onturn: {}", node.takestreak(), node.square().unwrap(), node.onturn());
        //println!("node.movecount(): {}, minmax.rootcount: {}", node.movecount(), minmax.rootcount);
        return e;
    }

    let moves = legal_moves(&node);
    // TODO sort the moves based on iterative deepening results
    // TODO add a table which saves the order of children nodes
    // TODO killer heuristic

    let mut best = eval::Eval::MIN;
    let mut betacut = false;

    for (square, cell) in moves {
        let mut child = node.clone();
        child.play(square, cell);

        best = std::cmp::max(
            best,
            negamax(&child, beta.rev(), alpha.rev(), minmax)?.rev(),
        );

        alpha = std::cmp::max(alpha, best);
        if alpha >= beta {
            betacut = true;
            break;
        }
    }

    if let Some(entry) = minmax.table.get(node.key())  {
        let table_value = eval_absolute(&entry.value, node.movecount() - minmax.rootcount);
        let table_value_original = eval_absolute(&entry.value, entry.movecount - minmax.rootcount);

        if alpha_original < best && best < beta && alpha_original < table_value && table_value < beta {
            match entry.flag {
                table::Flag::EXACT if best != table_value => {
                    minmax.stats.correct = false;
                    print!("\nboard: \n{}", node);
                    println!("invalid exact, no betacut: {} ", !betacut);
                    println!("actual value: {}, != table_value: {} / table_value_original {}", best, table_value, table_value_original);
                    println!("alpha old: {}, alpha: {}, beta: {}", alpha_original, alpha, beta);
                    println!("entry.alpha: {}, entry.beta: {}", entry.alpha, entry.beta);
                    println!("movecount: {}, entry.movecount: {}, rootcount: {}", node.movecount(), entry.movecount, minmax.rootcount);
                    println!("takestreak: {}, square: {}, onturn: {}", node.takestreak(), node.square().unwrap(), node.onturn());
                }
                table::Flag::LOWERBOUND if best < table_value => {
                    minmax.stats.correct = false;
                    print!("\nboard: \n{}", node);
                    println!("invalid lowerbound, no betacut: {} ", !betacut);
                    println!("actual value: {}, < table_value: {} / table_value_original {}", best, table_value, table_value_original);
                    println!("alpha old: {}, alpha: {}, beta: {}", alpha_original, alpha, beta);
                    println!("entry.alpha: {}, entry.beta: {}", entry.alpha, entry.beta);
                    println!("movecount: {}, entry.movecount: {}, rootcount: {}", node.movecount(), entry.movecount, minmax.rootcount);
                    println!("takestreak: {}, square: {}, onturn: {}", node.takestreak(), node.square().unwrap(), node.onturn());
                }
                table::Flag::UPPERBOUND if best > table_value => {
                    minmax.stats.correct = false;
                    print!("\nboard: \n{}", node);
                    println!("invalid upperbound, no betacut: {} ", !betacut);
                    println!("actual value: {}, > table_value: {} / table_value_original {}", best, table_value, table_value_original);
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
    //println!("\nsaving board in table: {} {:?}\n{}", best, flag, node);

    //println!("\nboard: \n{}", node);
    //println!("alpha old: {}, alpha: {}, beta: {}, best: {}", alpha_original, alpha, beta, best);
    //println!("takestreak: {}, square: {}, onturn: {}", node.takestreak(), node.square().unwrap(), node.onturn());
    //println!("best.distance: {}, node.movecount(): {}, minmax.rootcount: {}", best.distance, node.movecount(), minmax.rootcount);
    let table_best = eval_relative(&best, node.movecount() - minmax.rootcount);
    minmax.table.put(node.key(), table_best, flag, 0, alpha_original, beta, node.movecount()); // TODO add bestmove

    Ok(best)
}
