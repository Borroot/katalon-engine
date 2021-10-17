use crate::player::Players::{self, *};
use std::{fmt, result};

#[derive(Debug, PartialEq)]
pub enum Result {
    /// Use this if player1 has won.
    Player1,
    /// Use this if player2 has won.
    Player2,
    /// Use this if it is a draw.
    Draw,
    /// Use this if the game is not over yet.
    None,
}

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
    movecount: u16,
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
    pub fn load(moves: &str) -> result::Result<Self, String> {
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
            if board.canplay(c) {
                board.play(c);
            } else {
                return Err(format!(
                    "Move {} is invalid: ({}, {}).",
                    index + 1,
                    board.lastmove.1,
                    c
                ));
            }
        }
        return Ok(board);
    }

    /// Return Some((square, cell)) if double cell is given, otherwise return None.
    fn double(square: u8, cell: u8) -> Option<(u8, u8)> {
        match (square, cell) {
            (0, 4) => Some((2, 0)),
            (1, 3) => Some((2, 1)),
            (3, 1) => Some((2, 3)),
            (4, 0) => Some((2, 4)),

            (2, 0) => Some((0, 4)),
            (2, 1) => Some((1, 3)),
            (2, 3) => Some((3, 1)),
            (2, 4) => Some((4, 0)),

            _ => None,
        }
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

        // If the cell contains a stone of yourself, return false.
        if self.state & bit != 0 {
            return false;
        }

        let mask_square = 0b11111 << square * 5;

        // If the square is not full, return false.
        if self.mask & mask_square != mask_square {
            return false;
        }

        // If the cell is not equal to the lastmove, return true.
        // Also check the double cell connected to the lastmove.
        if self.lastmove.0 != square || self.lastmove.1 != cell {
            if let Some(lastdouble) = Board::double(self.lastmove.0, self.lastmove.1) {
                if lastdouble.0 != square || lastdouble.1 != cell {
                    return true;
                }
            } else {
                return true;
            }
        }

        // If there are no other possible moves, return true, else return false.
        return (self.state ^ bit) & mask_square == mask_square;
    }

    pub fn canplay(&self, cell: u8) -> bool {
        debug_assert!(self.lastmove.0 < 5 && self.lastmove.1 < 5);
        self.canplay_explicit(self.lastmove.1, cell)
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

        // Update the state and mask variable according to the move.
        let mut update = |square: u8, cell: u8| {
            let bit = 1 << square * 5 + cell;
            self.state ^= bit;
            self.mask |= bit;
        };

        // Update the state and mask of the cell.
        update(square, cell);

        // Update the double cell if we are in one.
        if let Some((s, c)) = Board::double(square, cell) {
            update(s, c);
        }

        // Update the stones, player onturn, state, lastmove and movecount.
        self.stones[self.onturn as usize] -= 1;
        self.onturn = self.onturn.other();
        self.state ^= self.mask;
        self.lastmove = (square, cell);
        self.movecount += 1;
    }

    pub fn play(&mut self, cell: u8) {
        self.play_explicit(self.lastmove.1, cell);
    }

    /// Check if the game is over.
    /// This function assumes the game was not over before the lastmove.
    pub fn isover(&self) -> Result {
        // Convert the onturn player to a result player type.
        let result = |player: Players| -> Result {
            match player {
                Player1 => Result::Player1,
                Player2 => Result::Player2,
            }
        };

        // No one can win within just 8 moves, at least 9 are needed.
        if self.movecount <= 8 {
            return Result::None;
        }

        // Check if the (previous) player has finished a square.
        if (self.mask ^ (0b11111 << self.lastmove.0 * 5)) | self.state == 0 {
            return result(self.onturn.other());
        }

        // Check if the board is full and if so who won.
        if self.mask == 0b11111_11111_11111_11111_11111 {
            let square_count_onturn = {
                (0u8..5u8)
                    .filter(|square| {
                        // Count the number of cells in the square.
                        (0u8..5u8)
                            .filter(|cell| self.state & 1 << square * 5 + cell != 0)
                            .count()
                            > 2
                    })
                    .count()
            };
            match square_count_onturn {
                c if c > 2 => return result(self.onturn),
                _ => return result(self.onturn.other()),
            }
        }

        // The streak of consecutively taking stones is reached.
        if self.takestreak == 20 {
            return Result::Draw;
        }

        // Check if the player onturn still has stones left.
        if self.stones[self.onturn as usize] == 0 {
            return result(self.onturn);
        }

        // The game is not over yet.
        return Result::None;
    }

    pub fn movecount(&self) -> u16 {
        self.movecount
    }

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
    fn load_basic() {
        assert!(Board::load("jfkd").is_err());
        assert!(Board::load("3").is_err());
        assert!(Board::load("35").is_err());
        assert!(Board::load("012345").is_err());

        assert!(Board::load("").is_ok());
        assert!(Board::load("02").is_ok());
        assert!(Board::load("01234").is_ok());
    }

    #[test]
    fn load_more() {
        let board1 = Board::load("0123432100304022").unwrap();

        assert_eq!(board1.state, 0b00000_10100_00001_00101_11100);
        assert_eq!(board1.mask, 0b01001_10111_11111_01101_11111);
        assert_eq!(board1.onturn, Player2);
        assert_eq!(board1.stones, [4, 5]);
        assert_eq!(board1.lastmove, (2, 2));
        assert_eq!(board1.takestreak, 0);
        assert_eq!(board1.movecount, 15);

        let board2 = Board::load("01234321003040223").unwrap();

        assert_eq!(board2.state, 0b01001_00001_10110_01000_00011);
        assert_eq!(board2.mask, 0b01001_10111_11111_01101_11111);
        assert_eq!(board2.onturn, Player1);
        assert_eq!(board2.stones, [5, 4]);
        assert_eq!(board2.lastmove, (2, 3));
        assert_eq!(board2.takestreak, 1);
        assert_eq!(board2.movecount, 16);

        // TODO add test case which errors because game ended
    }

    #[test]
    fn play_empty() {
        let mut board = Board::new();
        board.play_explicit(3, 4);

        assert_eq!(board.state, 0b00000_00000_00000_00000_00000);
        assert_eq!(board.mask, 0b00000_10000_00000_00000_00000);
        assert_eq!(board.onturn, Player2);
        assert_eq!(board.stones, [11, 12]);
        assert_eq!(board.lastmove, (3, 4));
        assert_eq!(board.takestreak, 0);
        assert_eq!(board.movecount, 1);

        board.play(1);

        assert_eq!(board.state, 0b00000_10000_00000_00000_00000);
        assert_eq!(board.mask, 0b00010_10000_00000_00000_00000);
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
        assert_eq!(board.mask, 0b00000_00001_00001_00001_11111);
        assert_eq!(board.stones, [9, 8]);
        assert_eq!(board.takestreak, 1);

        board.play(3);

        assert_eq!(board.state, 0b00000_00000_00000_00000_00111);
        assert_eq!(board.mask, 0b00000_00001_00001_00001_11111);
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
        assert_eq!(board1.mask, 0b00000_00000_00001_00000_10000);

        let mut board2 = Board::new();
        board2.play_explicit(1, 3);
        assert_eq!(board2.mask, 0b00000_00000_00010_01000_00000);

        let mut board3 = Board::new();
        board3.play_explicit(3, 1);
        assert_eq!(board3.mask, 0b00000_00010_01000_00000_00000);

        let mut board4 = Board::new();
        board4.play_explicit(4, 0);
        assert_eq!(board4.mask, 0b00001_00000_10000_00000_00000);

        let mut board5 = Board::new();
        board5.play_explicit(2, 0);
        assert_eq!(board5.mask, 0b00000_00000_00001_00000_10000);

        let mut board6 = Board::new();
        board6.play_explicit(2, 1);
        assert_eq!(board6.mask, 0b00000_00000_00010_01000_00000);

        let mut board7 = Board::new();
        board7.play_explicit(2, 3);
        assert_eq!(board7.mask, 0b00000_00010_01000_00000_00000);

        let mut board8 = Board::new();
        board8.play_explicit(2, 4);
        assert_eq!(board8.mask, 0b00001_00000_10000_00000_00000);
    }

    /// Test playing in an empty cell.
    #[test]
    fn canplay_empty() {
        let mut board = Board::load("00").unwrap();

        assert!(!board.canplay(0));

        assert!(board.canplay(1));
        assert!(board.canplay(2));
        assert!(board.canplay(3));
        assert!(board.canplay(4));

        board.play(4);
        board.play(0);
        board.play(2);

        assert!(!board.canplay(0));
        assert!(!board.canplay(4));

        assert!(board.canplay(1));
        assert!(board.canplay(2));
        assert!(board.canplay(3));

        board.play(2);

        assert!(!board.canplay(0));
        assert!(!board.canplay(2));
        assert!(!board.canplay(4));

        assert!(board.canplay(1));
        assert!(board.canplay(3));
    }

    /// Test taking a stone.
    #[test]
    fn canplay_takes_normal() {
        let mut board1 = Board::load("00203010").unwrap();

        assert!(board1.canplay(0));
        assert!(board1.canplay(4));

        assert!(!board1.canplay(1));
        assert!(!board1.canplay(2));
        assert!(!board1.canplay(3));

        board1.play(0);

        assert!(board1.canplay(1));
        assert!(board1.canplay(2));
        assert!(board1.canplay(3));

        assert!(!board1.canplay(0));
        assert!(!board1.canplay(4));

        board1.play(2);

        assert!(board1.canplay(1));
        assert!(board1.canplay(2));
        assert!(board1.canplay(3));
        assert!(board1.canplay(4));

        assert!(!board1.canplay(0));

        let board2 = Board::load("11210141").unwrap();

        assert!(board2.canplay(1));
        assert!(board2.canplay(3));

        assert!(!board2.canplay(0));
        assert!(!board2.canplay(2));
        assert!(!board2.canplay(4));
    }

    /// Test taking the previous move.
    #[test]
    fn canplay_takes_prev() {
        // normal cannot take
        let board1 = Board::load("12101411").unwrap();
        assert!(!board1.canplay(1));

        // double cannot take
        let board2 = Board::load("442343214122024").unwrap();
        assert!(board2.canplay(2));
        assert!(!board2.canplay(0));

        // we can take
        let board3 = Board::load("24232021122").unwrap();
        assert!(board3.canplay(2));
    }

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
