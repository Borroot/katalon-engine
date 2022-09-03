use super::table;
use crate::{board, eval, player, stats::search};

/// Various variables needed during the negamax search.
pub struct Negamax {
    /// A receiving thread for when the timeout is reached.
    recv_timeout: std::sync::mpsc::Receiver<()>,
    /// The movecount when the search did not start yet.
    rootcount: i16,
    /// The player who is on turn when the search starts.
    rootplayer: player::Players,
    /// The transposition table used by the negamax search.
    pub table: table::Table,
    /// Statistics on the negamax search.
    pub stats: search::Stats,
}

impl Negamax {
    /// Create new variables needed for a fresh search.
    pub fn new(
        recv_timeout: std::sync::mpsc::Receiver<()>,
        rootcount: i16,
        rootplayer: player::Players,
    ) -> Self {
        Self {
            recv_timeout,
            rootcount,
            rootplayer,
            // TODO make gb adaptive to movecount
            table: table::Table::from_gb(1.0),
            stats: search::Stats::new(),
        }
    }
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

pub fn eval(
    node: &board::Board,
    mut alpha: eval::Eval,
    mut beta: eval::Eval,
    negamax: &mut Negamax,
) -> Result<eval::Eval, ()> {
    debug_assert!(alpha < beta);

    // Check if the timeout is reached and we should interrupt the search.
    let recv = negamax.recv_timeout.try_recv();
    if recv.is_ok() || recv == Err(std::sync::mpsc::TryRecvError::Disconnected) {
        negamax.stats.timeout = true;
        return Err(());
    }

    negamax.stats.visited += 1;
    let alpha_original = alpha;

    // TODO also check symmetries if depth is low and make a seperate hashmap for low movecount evaluations.
    // Check if we have already seen this node before.
    if let Some(entry) = negamax.table.get(node.key()) {
        let table_value = entry.value.absolute(negamax.rootcount, node.movecount());
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
            negamax.rootplayer,
            node.movecount() - negamax.rootcount,
        ));
    }

    // Generate and sort the moves. Put cells that go to full squares up front.
    let mut moves = node.moves();
    moves.sort_by(|(_s1, c1), (_s2, _c2)| match node.isfull(*c1) {
        true => std::cmp::Ordering::Greater,
        false => std::cmp::Ordering::Less,
    });

    // Do the search recursive over all the child nodes.
    let mut value = eval::Eval::MIN;

    for (square, cell) in moves {
        let mut child = node.clone();
        child.play(square, cell);

        value = std::cmp::max(value, -eval(&child, -beta, -alpha, negamax)?);

        alpha = std::cmp::max(alpha, value);
        if alpha >= beta {
            break;
        }
    }

    // Save the current state and the evaluation in the table.
    let flag = {
        if value <= alpha_original {
            table::Flag::UPPERBOUND
        } else if value >= beta {
            table::Flag::LOWERBOUND
        } else {
            table::Flag::EXACT
        }
    };
    let table_value = value.relative(negamax.rootcount, node.movecount());
    negamax.table.put(node.key(), table_value, flag);

    Ok(value)
}
