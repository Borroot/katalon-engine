use crate::{board, player};
use rand::prelude::*;
use std::cmp;

/// A player directed by the minmax algorithm.
pub struct Minmax;

impl Minmax {
    /// Return the humanized evaluation of the given board.
    pub fn evaluate(board: &board::Board) -> isize {
        Minmax::humanize_relative(
            board.movecount() as isize,
            negamax(board, -isize::MAX, isize::MAX, 1, &board.onturn()),
        )
    }

    /// E.g. -25 if loss on the 25th move and 10 if win on the 10th move.
    fn humanize_absolute(value: isize) -> isize {
        match value {
            v if v < 0 => -(value + isize::MAX),
            v if v > 0 => -(value - isize::MAX),
            _ => 0,
        }
    }

    /// E.g. -8 if loss in 8 moves and 10 if win in 10 moves.
    fn humanize_relative(movecount: isize, value: isize) -> isize {
        match value {
            v if v < 0 => Minmax::humanize_absolute(value) + movecount,
            v if v > 0 => Minmax::humanize_absolute(value) - movecount,
            _ => 0,
        }
    }
}

impl player::Player for Minmax {
    fn play(&self, node: &board::Board) -> (u8, u8) {
        let mut bestmoves: Vec<(u8, u8)> = Vec::new();
        let mut max = isize::MIN;

        let me = node.onturn();

        // Generate and sort the children
        let moves = moves(&node);
        // TODO sort

        for (square, cell) in moves {
            let mut child = node.clone();
            child.play(square, cell);

            let value = -negamax(&child, -isize::MAX, isize::MAX, -1, &me);

            if value > max {
                max = value;
                bestmoves.clear();
                bestmoves.push((square, cell));
            } else if value == max {
                bestmoves.push((square, cell));
            }
        }

        let mut rng = rand::thread_rng();
        bestmoves[rng.gen_range(0..bestmoves.len()) as usize]
    }
}

/// Evaluate the board from the perspective of the root player.
/// Return  isize::MAX - movecount if won
/// Return -isize::MAX + movecount if loss
/// Return 0 if draw.
fn evaluate(result: &board::Result, root: &player::Players, movecount: u16) -> isize {
    match result.player() {
        Some(player) => {
            if player == *root {
                return isize::MAX - movecount as isize;
            } else {
                return -isize::MAX + movecount as isize;
            }
        }
        None => return 0, // draw
    }
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
    mut alpha: isize,
    beta: isize,
    color: isize,
    root: &player::Players,
) -> isize {
    // Compute the value of the leaf node
    let result = node.isover();
    if result != None {
        return color * evaluate(&result.unwrap(), root, node.movecount());
    }

    // Generate and sort the children
    let moves = moves(&node);
    // TODO sort

    let mut value = isize::MIN;

    for (square, cell) in moves {
        let mut child = node.clone();
        child.play(square, cell);

        value = cmp::max(value, -negamax(&child, -beta, -alpha, -color, root));
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
        assert_eq!(Minmax.play(&board1), (2, 2));
        assert_eq!(Minmax::evaluate(&board1), 1);

        // As player2, depth 1
        let board2 = board::Board::load("0020103040").unwrap();
        assert_eq!(Minmax.play(&board2), (0, 0));
        assert_eq!(Minmax::evaluate(&board2), 1);

        // As player1, depth 2
        let board3 = board::Board::load("01234321042244114110033").unwrap();
        assert_eq!(Minmax::evaluate(&board3), 2);
    }

    #[test]
    fn win_two_options() {
        // As player2, depth 3
        let board1 = board::Board::load("2200103024131211424323").unwrap();
        assert_eq!(Minmax.play(&board1), (3, 3));
        assert_eq!(Minmax::evaluate(&board1), 3);
    }

    /// Test if minmax detects its gonna lose.
    #[test]
    fn loss_one_option() {
        // As player1, depth 2
        let board1 = board::Board::load("22001030241312114243233").unwrap();
        assert_eq!(Minmax.play(&board1), (3, 4));
        assert_eq!(Minmax::evaluate(&board1), -2);
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
        assert_eq!(Minmax::evaluate(&board), 0);
    }
}
