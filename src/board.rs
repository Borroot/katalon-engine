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

            movecount: 0,
        }
    }

    /// Create a board with the specified configuration.
    pub fn load(moves: &str) -> Result<Self, String> {
        let mut board = Self::new();

        if !moves.chars().all(|c| '0' <= c && c <= '4') {
            return Err("Please only use the digits 0 to 4.".to_string());
        }

        if moves.len() == 0 {
            return Ok(board);
        } else if moves.len() == 1 {
            return Err("Please provide the square for the first move.".to_string());
        }

        let mut cs = moves.chars().map(|c: char| c as u8 - '0' as u8);
        board.play_explicit(cs.next().unwrap(), cs.next().unwrap());

        for (index, c) in cs.enumerate() {
            // TODO add isfinalmove check
            if !board.canplay(c) {
                return Err(format!("Move {} is invalid: ({}, {}).", index + 1, board.lastmove.0, c));
            } else {
                board.play(c);
            }
        }
        return Ok(board);
    }

    pub fn canplay_explicit(&self, square: u8, cell: u8) -> bool {
        debug_assert!(self.lastmove.1 == 0xFF || self.lastmove.1 == square);
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
        if self.lastmove.0 == square && self.lastmove.1 == cell {
            if (self.state ^ bit) & 0b11111 << square * 5 != 0b11111 {
                return true;
            } else {
                return false;
            }
        }

        // We can take an opponents stone (!= lastmove).
        return true;
    }

    pub fn canplay(&self, cell: u8) -> bool {
        debug_assert!(self.lastmove.0 < 5 && self.lastmove.1 < 5);
        self.canplay_explicit(self.lastmove.1, cell)
    }

    /// Update the state and mask variable according to the move.
    fn update(&mut self, square: u8, cell: u8) {
        let bit = 1 << square * 5 + cell;
        self.state ^= bit;
        self.mask |= bit;
    }

    pub fn play_explicit(&mut self, square: u8, cell: u8) {
        debug_assert!(self.canplay_explicit(square, cell));

        // Check if we take a stone from the opponent.
        if self.mask & 1 << square * 5 + cell != 0 {
            self.stones[self.onturn.other() as usize] += 1;
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
        self.stones[self.onturn as usize] -= 1;
        self.onturn = self.onturn.other();
        self.state ^= self.mask;
        self.lastmove = (square, cell);
        self.movecount += 1;
    }

    pub fn play(&mut self, cell: u8) {
        self.play_explicit(self.lastmove.1, cell);
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

    #[test]
    fn load() {
        assert!(Board::load("jfkd").is_err());
        assert!(Board::load("3").is_err());
        assert!(Board::load("35").is_err());
        assert!(Board::load("012345").is_err());

        assert!(Board::load("").is_ok());
        assert!(Board::load("02").is_ok());
        assert!(Board::load("01234").is_ok());
        assert!(Board::load("01234321003040223").is_ok());

        let board = Board::load("01234321003040223").unwrap();
        assert_eq!(board.state, 0b01000_00000_00011_01000_11110);
        assert_eq!(board.mask,  0b01001_11111_11111_01101_11111);
        assert_eq!(board.onturn, Player1);
        assert_eq!(board.stones, [6, 2]);
        assert_eq!(board.lastmove, (3, 3));
        assert_eq!(board.takestreak, 1);
        assert_eq!(board.movecount, 18);

        // TODO add test case which errors because game ended
    }

    #[test]
    fn play_empty() {
        let mut board = Board::new();
        board.play_explicit(3, 4);

        assert_eq!(board.state, 0b00000_00000_00000_00000_00000);
        assert_eq!(board.mask,  0b00000_10000_00000_00000_00000);
        assert_eq!(board.onturn, Player2);
        assert_eq!(board.stones, [11, 12]);
        assert_eq!(board.lastmove, (3, 4));
        assert_eq!(board.takestreak, 0);
        assert_eq!(board.movecount, 1);

        board.play(1);

        assert_eq!(board.state, 0b00000_10000_00000_00000_00000);
        assert_eq!(board.mask,  0b00010_10000_00000_00000_00000);
        assert_eq!(board.onturn, Player1);
        assert_eq!(board.stones, [11, 11]);
        assert_eq!(board.lastmove, (4, 1));
        assert_eq!(board.takestreak, 0);
        assert_eq!(board.movecount, 2);
    }

    #[test]
    fn play_takes() {
        let mut board = Board::load("00203010").unwrap();
        board.play(0);

        assert_eq!(board.state, 0b00000_00001_00001_00001_10000);
        assert_eq!(board.mask,  0b00000_00001_00001_00001_11111);
        assert_eq!(board.stones, [9, 8]);
        assert_eq!(board.takestreak, 1);

        board.play(3);

        assert_eq!(board.state, 0b00000_00001_00001_00001_11000);
        assert_eq!(board.mask,  0b00000_00001_00001_00001_11111);
        assert_eq!(board.stones, [8, 9]);
        assert_eq!(board.takestreak, 2);

        board.play(4);

        assert_eq!(board.takestreak, 0);
    }

    #[test]
    fn play_double() {
        let mut board1 = Board::new();
        board1.play_explicit(0, 4);
        assert_eq!(board1.state, 0b00000_00000_00000_00000_00000);
        assert_eq!(board1.mask,  0b00000_00000_00001_00000_10000);

        let mut board2 = Board::new();
        board2.play_explicit(1, 3);
        assert_eq!(board2.mask,  0b00000_00000_00010_01000_00000);

        let mut board3 = Board::new();
        board3.play_explicit(3, 1);
        assert_eq!(board3.mask,  0b00000_00010_01000_00000_00000);

        let mut board4 = Board::new();
        board4.play_explicit(4, 0);
        assert_eq!(board4.mask,  0b00001_00000_10000_00000_00000);

        let mut board5 = Board::new();
        board5.play_explicit(2, 0);
        assert_eq!(board5.mask,  0b00000_00000_00001_00000_10000);

        let mut board6 = Board::new();
        board6.play_explicit(2, 1);
        assert_eq!(board6.mask,  0b00000_00000_00010_01000_00000);

        let mut board7 = Board::new();
        board7.play_explicit(2, 3);
        assert_eq!(board7.mask,  0b00000_00010_01000_00000_00000);

        let mut board8 = Board::new();
        board8.play_explicit(2, 4);
        assert_eq!(board8.mask,  0b00001_00000_10000_00000_00000);
    }

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
