use crate::player::Players::{self, *};
use std::{fmt, result::Result};

#[derive(Debug)]
pub struct Board {
    /// The places at which the player onturn has a stone.
    /// 0b00000_00000_00000_00000_00000
    ///   sqr4  sqr3  sqr2  sqr1  sqr0
    state: u32,

    /// The places at which either player has a stone.
    /// 0b00000_00000_00000_00000_00000
    ///   sqr4  sqr3  sqr2  sqr1  sqr0
    mask: u32,

    /// The player which is currently on turn.
    onturn: Players,

    /// The number of stones left for the players (player1, player2).
    /// Both players start with 12 stones.
    stones: [u8; 2],

    /// The last move that was made. Can be used to get the square constraint.
    /// This piece cannot be taken, unless it is the only option.
    lastmove: (u8, u8),

    /// Keeps how many turns in a row pieces have been taken.
    takestreak: u8,

    /// The number of moves that have been made.
    movecount: u8,
}

impl Board {
    /// Create a new empty board.
    pub fn new() -> Self {
        Self {
            state: 0,
            mask: 0,
            onturn: Player1,
            stones: [12, 12],
            lastmove: (0xFF, 0xFF),
            takestreak: 0,
        }
    }

    /// Create a board with the specified configuration.
    pub fn load(moves: &String) -> Result<Self, String> {
        let mut board = Self::new();

        if !moves.chars().all(|c| '0' <= c && c <= '4') {
            return Err("Please only use the digits 0 to 4.");
        }

        let convert = |c: char| c - '0' as u8;
        board.play_explicit(convert(moves[0]), convert(moves[1]));

        for (index, c) in moves.chars().enumerate()[2..] {
            // TODO add isfinalmove check
            if !board.canplay() {
                return Err(format!("Move {} is invalid: ({}, {}).", first.0, first.1));
            } else {
                board.play(convert(c));
            }
        }
        return Ok(board);
    }

    pub fn canplay_explicit(&self, square: u8, cell: u8) -> bool {
        debug_assert!(self.lastmove.0 == 0xFF || self.lastmove.0 == square);
        debug_assert!(self.stones[self.onturn as usize] > 0);
        debug_assert!(square < 5 && cell < 5);

        let bit = 1 << square * 5 + cell;

        // If the cell is empty, return true. Most will return here.
        if self.mask & bit == 0 {
            return true;
        }

        // If the cell contains a stone of yourself
        // OR if the square is not full, return false.
        if self.state & bit != 0 || self.state & 0b11111 << square * 5 != 0b11111 {
            return false;
        }

        // If the cell is equal to the lastmove,
        // AND if there are no other possible moves, return true.
        if self.lastmove.0 == square && self.lastmove.1 == cell
            && self.state ^ bit & 0b11111 << square * 5 != 0b11111
        {
            return true;
        }

        // We can take an opponents stone (!= lastmove).
        return true;
    }

    pub fn canplay(&self, cell: u8) -> bool {
        debug_assert!(self.lastmove.0 < 5 && self.lastmove.1 < 5);
        self.canplay_explicit(self.lastmove.0, cell)
    }

    /// Update the state and mask variable according to the move.
    fn update(&mut self, square: u8, cell: u8) {
        let bit = 1 << square * 5 + cell;
        self.state ^= bit;
        self.state |= bit;
    }

    pub fn play_explicit(&mut self, square: u8, cell: u8) {
        debug_assert!(self.canplay_explicit(square, cell));

        // Check if we take a stone from the opponent.
        if self.mask & 1 << square * 5 + cell == 0 {
            self.stones[self.onturn.other() as usize] -= 1;
            self.stones[self.onturn as usize] += 1;

            self.takestreak += 1;
        } else if self.takestreak > 0 {
            self.takestreak = 0;
        }

        self.update(square, cell);

        // Check if we play in a double cell.
        match (square, cell) {
            (0, 4) => self.update(2, 0),
            (1, 3) => self.update(2, 1),
            (3, 1) => self.update(2, 3),
            (4, 0) => self.update(2, 4),

            (2, 0) => self.update(0, 4),
            (2, 1) => self.update(1, 3),
            (2, 3) => self.update(3, 1),
            (2, 4) => self.update(4, 0),

            _ => (),
        }

        // Update the player onturn, the state and the lastmove.
        self.onturn = self.onturn.other();
        self.state ^= self.mask;
        self.lastmove = (square, cell);
    }

    pub fn play(&mut self, cell: u8) {
        self.play_explicit(self.lastmove.0, cell);
    }

    // isfinalmove(move): the other player has no stones left after your turn
    // (you lose) you fill a square (you win), the taken_streak is reached
    // (draw), or the board is full (depends)

    // nbmoves(): returns move_count
    // hash(): small bit representation of the board

    fn symbol(&self, square: u8, cell: u8) -> String {
        let index = square * 5 + cell;
        if self.mask & 1 << index == 0 {
            ".".to_string()
        } else if self.state & 1 << index == 0 {
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

    // If the cell is empty, return true. Most will return here.

    // If the cell contains a stone of yourself
    // OR if the square is not full, return false.

    // If the cell is equal to the lastmove,
    // AND if there are no other possible moves, return true.

    // We can take an opponents stone (!= lastmove).

    /// Test playing in a wrong square.
    #[test]
    fn canplay_square() {}

    /// Test playing in an empty cell.
    #[test]
    fn canplay_empty() {}

    /// Test taking a stone.
    #[test]
    fn canplay_takes_normal() {}

    /// Test taking the previous move.
    #[test]
    fn canplay_takes_prev() {}

    /// Test display of nonempty board.
    #[test]
    fn display_nonempty() {
        let mut board = Board::new();

        board.state = 0b10101_10101_10101_10101_10101;
        board.mask = 0b11111_11111_11111_11111_11111;

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

    /// Test display of empty board.
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
