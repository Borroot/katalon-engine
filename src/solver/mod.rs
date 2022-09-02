use crate::{board, player};
use rand::Rng;

mod best;
mod eval;
mod negamax;
mod table;

// Make these function visible as e.g. solver::bestmoves.
pub use best::{bestmoves, bestmoves_with_stats};
pub use eval::{eval, eval_with_stats};//, eval_all};

/// A player directed by the negamax algorithm.
pub struct Solver;

impl player::Player for Solver {
    fn play(&self, node: &board::Board) -> (u8, u8) {
        let bestmoves = bestmoves(node, std::time::Duration::MAX).unwrap();
        let mut rng = rand::thread_rng();
        bestmoves[rng.gen_range(0..bestmoves.len()) as usize]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval;

    /// Test if negamax detects its gonna be a draw.
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
        assert_eq!(
            eval(&board, std::time::Duration::MAX).unwrap(),
            eval::Eval::from(eval::Result::Draw, 1)
        );
    }
}
