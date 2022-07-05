use crate::{board, eval, player};
use rand::Rng;

mod minmax;
mod table;

/// A player directed by the minmax algorithm.
pub struct Solver;

impl player::Player for Solver {
    fn play(&self, node: &board::Board) -> (u8, u8) {
        let bestmoves = bestmoves(node, std::time::Duration::MAX).unwrap().1;
        let mut rng = rand::thread_rng();
        bestmoves[rng.gen_range(0..bestmoves.len()) as usize]
    }
}

// TODO add a function which evaluates all the moves

/// Return all of the best moves if finished within the specified time with stats.
pub fn bestmoves_with_stats(
    node: &board::Board,
    timeout: std::time::Duration,
) -> (Result<(eval::Eval, Vec<(u8, u8)>), ()>, minmax::Stats) {
    let (send_timeout, recv_timeout) = std::sync::mpsc::channel();
    let (send_result, recv_result) = std::sync::mpsc::channel();

    let node_clone = node.clone();

    std::thread::spawn(move || {
        send_result
            .send(minmax::bestmoves(&node_clone, recv_timeout))
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
pub fn bestmoves(
    node: &board::Board,
    timeout: std::time::Duration,
) -> Result<(eval::Eval, Vec<(u8, u8)>), ()> {
    bestmoves_with_stats(node, timeout).0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::player::Player;

    // TODO remove these tests except for the last one

    fn evaluate(board: &board::Board) -> eval::Eval {
        bestmoves(board, std::time::Duration::MAX).unwrap().0
    }

    /// Test if minmax detects the winning play.
    #[test]
    fn win_one_option() {
        // As player1, depth 1
        let board1 = board::Board::load("202123242").unwrap();
        assert_eq!(Solver.play(&board1), (2, 2));
        assert_eq!(evaluate(&board1), eval::Eval::from(eval::Result::Win, 1));

        // As player2, depth 1
        let board2 = board::Board::load("0020103040").unwrap();
        assert_eq!(Solver.play(&board2), (0, 0));
        assert_eq!(evaluate(&board2), eval::Eval::from(eval::Result::Win, 1));

        // As player1, depth 2
        let board3 = board::Board::load("01234321042244114110033").unwrap();
        assert_eq!(evaluate(&board3), eval::Eval::from(eval::Result::Win, 2));
    }

    #[test]
    /// Test if minmax detects the winning plays.
    fn win_two_options() {
        // As player2, depth 3
        let board1 = board::Board::load("2200103024131211424323").unwrap();
        assert_eq!(Solver.play(&board1), (3, 3));
        assert_eq!(evaluate(&board1), eval::Eval::from(eval::Result::Win, 3));
    }

    /// Test if minmax detects its gonna lose.
    #[test]
    fn loss_one_option() {
        // As player1, depth 2
        let board1 = board::Board::load("22001030241312114243233").unwrap();
        assert_eq!(Solver.play(&board1), (3, 4));
        assert_eq!(evaluate(&board1), eval::Eval::from(eval::Result::Loss, 2));
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
        assert_eq!(evaluate(&board), eval::Eval::from(eval::Result::Draw, 1));
    }
}
