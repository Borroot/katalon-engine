use crate::player::Players::{self, *};
use std::fmt;

#[derive(Debug)]
pub struct Board {
    state: u32,
    mask: u32,

    /// The player which is currently on turn.
    onturn: Players,

    // stones_left (player1, player2): stones left for player 1

    // square_constraint: you can only move in this square
    // previous_move: this piece cannot be taken, unless it is the only option
    // taken_streak: if last 10 turns only pieces are taken, then its a draw

    // move_count: number of moves made
}

impl Board {
    pub fn new() -> Self {
        Self {
            state: 0,
            mask: 0,
            onturn: Player1,
        }
    }

    // new(moves)
    // new()

    // canplay(move)
    // play(move)
    // isfinalmove(move): the other player has no stones left after your turn
    // (you lose) you fill a square (you win), the taken_streak is reached
    // (draw), or the board is full (depends)

    // nbmoves(): returns move_count
    // hash(): small bit representation of the board

    fn symbol(&self, square: u8, cell: u8) -> String {
        if self.mask & 1 << square * 5 + cell == 0 {
            ".".to_string()
        } else if self.state & 1 << square * 5 + cell == 0 {
            self.onturn.other().to_string()
        } else {
            self.onturn.to_string()
        }
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            concat!(
                "+---------------------------+\n",
                "| {}       {} |   | {}       {} |\n",
                "|           |   |           |\n",
                "|     {}     |   |     {}     |\n",
                "|       +---+---+---+       |\n",
                "| {}     | {} |   | {} |     {} |\n",
                "+-------+---+   +---+-------+\n",
                "|       |     {}     |       |\n",
                "+-------+---+   +---+-------+\n",
                "| {}     | {} |   | {} |     {} |\n",
                "|       +---+---+---+       |\n",
                "|     {}     |   |     {}     |\n",
                "|           |   |           |\n",
                "| {}       {} |   | {}       {} |\n",
                "+---------------------------+\n",
            ),
            self.symbol(0, 0),
            self.symbol(0, 1),
            self.symbol(1, 0),
            self.symbol(1, 1),
            self.symbol(0, 2),
            self.symbol(1, 2),
            self.symbol(0, 3),
            self.symbol(0, 4),
            self.symbol(1, 3),
            self.symbol(1, 4),
            self.symbol(2, 2),
            self.symbol(3, 0),
            self.symbol(3, 1),
            self.symbol(4, 0),
            self.symbol(4, 1),
            self.symbol(3, 2),
            self.symbol(4, 2),
            self.symbol(3, 3),
            self.symbol(3, 4),
            self.symbol(4, 3),
            self.symbol(4, 4)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_nonempty() {
        let mut board = Board::new();

        board.state = 0b10101_10101_10101_10101_10101;
        board.mask =  0b11111_11111_11111_11111_11111;

        assert_eq!(
            format!("{}", board),
            concat!(
                "+---------------------------+\n",
                "| X       O |   | X       O |\n",
                "|           |   |           |\n",
                "|     X     |   |     X     |\n",
                "|       +---+---+---+       |\n",
                "| O     | X |   | O |     X |\n",
                "+-------+---+   +---+-------+\n",
                "|       |     X     |       |\n",
                "+-------+---+   +---+-------+\n",
                "| X     | O |   | X |     O |\n",
                "|       +---+---+---+       |\n",
                "|     X     |   |     X     |\n",
                "|           |   |           |\n",
                "| O       X |   | O       X |\n",
                "+---------------------------+\n",
            )
        );
    }

    #[test]
    fn display_empty() {
        assert_eq!(
            format!("{}", Board::new()),
            concat!(
                "+---------------------------+\n",
                "| .       . |   | .       . |\n",
                "|           |   |           |\n",
                "|     .     |   |     .     |\n",
                "|       +---+---+---+       |\n",
                "| .     | . |   | . |     . |\n",
                "+-------+---+   +---+-------+\n",
                "|       |     .     |       |\n",
                "+-------+---+   +---+-------+\n",
                "| .     | . |   | . |     . |\n",
                "|       +---+---+---+       |\n",
                "|     .     |   |     .     |\n",
                "|           |   |           |\n",
                "| .       . |   | .       . |\n",
                "+---------------------------+\n",
            )
        );
    }
}