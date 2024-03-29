use crate::{player, player::Player, random};
use rand::Rng;

mod key;

#[derive(Debug, Eq, PartialEq)]
pub enum Result {
    /// Use this if player1 has won.
    Player1,
    /// Use this if player2 has won.
    Player2,
    /// Use this if it is a draw.
    Draw,
}

impl Result {
    pub fn from_player(player: &player::Players) -> Self {
        match player {
            player::Players::Player1 => Result::Player1,
            player::Players::Player2 => Result::Player2,
        }
    }

    pub fn player(&self) -> Option<player::Players> {
        match self {
            Result::Player1 => Some(player::Players::Player1),
            Result::Player2 => Some(player::Players::Player2),
            Result::Draw => None,
        }
    }
}

#[derive(Debug, Clone)]
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
    onturn: player::Players,

    /// The number of stones left for the players (player1, player2).
    /// Both players start with NUMBER_OF_STONES stones.
    stones: [u8; 2],

    /// The last move that was made (square, cell). Used to get the square
    /// constraint. This piece cannot be taken, unless it is the only option.
    lastmove: Option<(u8, u8)>,

    /// Keeps how many turns in a row pieces have been taken.
    takestreak: u8,

    /// The number of moves that have been made.
    // This is an i16 because this make eval::Eval more efficient.
    movecount: i16,
}

impl Board {
    /// The maximum number of takes that are allowed to be made in a row.
    // Make sure this number fits in 7 bits so it fits in the key.
    pub const TAKESTREAK_LIMIT: u8 = 15; // <= 127

    /// An upperbound on the maximum number of moves that can be made in a game.
    pub const MOVECOUNT_LIMIT: i16 = (21 - 7) * Self::TAKESTREAK_LIMIT as i16;

    /// The number of stones each player starts with.
    pub const NUMBER_OF_STONES: u8 = 12;

    /// Create a new empty board.
    pub fn new() -> Self {
        Self {
            state: 0,
            mask: 0,

            onturn: player::Players::Player1,
            stones: [Self::NUMBER_OF_STONES, Self::NUMBER_OF_STONES],

            lastmove: None,
            takestreak: 0,

            movecount: 0,
        }
    }

    /// Create a board with the specified configuration.
    pub fn load(moves: &str) -> std::result::Result<Self, String> {
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
        board.play(cs.next().unwrap(), cs.next().unwrap());

        let mut gamefinished = false;
        for c in cs {
            if gamefinished {
                return Err(format!(
                    "Move {} is invalid: ({}, {}), game finished.",
                    board.movecount() + 1,
                    board.lastmove.unwrap().1,
                    c
                ));
            } else if board.canplay(board.square().unwrap(), c) {
                board.play(board.square().unwrap(), c);
                gamefinished = board.isover() != None;
            } else {
                return Err(format!(
                    "Move {} is invalid: ({}, {}).",
                    board.movecount() + 1,
                    board.lastmove.unwrap().1,
                    c
                ));
            }
        }
        return Ok(board);
    }

    /// Create a random board position which may or may not be game over.
    pub fn random() -> Self {
        let mut rng = rand::thread_rng();
        let player = random::Random;

        let mut board = Self::new();

        for _ in 0..rng.gen_range(0..=Board::MOVECOUNT_LIMIT) {
            let (square, cell) = player.play(&board);
            board.play(square, cell);

            if board.isover() != None {
                break;
            }
        }
        board
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

            (_, _) => None,
        }
    }

    /// Check if the onturn player can play on (square, cell).
    pub fn canplay(&self, square: u8, cell: u8) -> bool {
        debug_assert!(self.lastmove == None || self.lastmove.unwrap().1 == square);
        debug_assert!(self.stones[self.onturn as usize] > 0);
        debug_assert!(square < 5 && cell < 5);
        debug_assert!(self.isover() == None);

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
        if self.lastmove.unwrap().0 != square || self.lastmove.unwrap().1 != cell {
            match Self::double(self.lastmove.unwrap().0, self.lastmove.unwrap().1) {
                None => return true,
                Some((s, c)) if s != square || c != cell => return true,
                Some(_) => (), // double == lastmove
            }
        }

        // If there are no other possible moves, return true, else return false.
        return (self.state ^ bit) & mask_square == mask_square;
    }

    /// Make the play at the given (square, cell).
    pub fn play(&mut self, square: u8, cell: u8) {
        debug_assert!(self.canplay(square, cell));

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
        if let Some((s, c)) = Self::double(square, cell) {
            update(s, c);
        }

        // Update the stones, player onturn, state, lastmove and movecount.
        self.stones[self.onturn as usize] -= 1;
        self.onturn = self.onturn.other();
        self.state ^= self.mask;
        self.lastmove = Some((square, cell));
        self.movecount += 1;
    }

    /// Check if the game is over, as a result of the lastmove!
    pub fn isover(&self) -> Option<Result> {
        // No one can win within just 8 moves, at least 9 are needed.
        if self.movecount <= 8 {
            return None;
        }

        // Check if the given square is finished by the previous player.
        let check_square = |square: u8| -> bool {
            let mask_square = 0b11111 << square * 5;
            return self.mask & mask_square & (self.state ^ mask_square) == mask_square;
        };

        // Check if the (previous) player has finished a square.
        // Also check the double square if applicable.
        if check_square(self.lastmove.unwrap().0) {
            return Some(Result::from_player(&self.onturn.other()));
        }
        if let Some((square, _)) = Self::double(self.lastmove.unwrap().0, self.lastmove.unwrap().1)
        {
            if check_square(square) {
                return Some(Result::from_player(&self.onturn.other()));
            }
        }

        // Check if the board is full and if so who won.
        if self.mask == 0b11111_11111_11111_11111_11111 {
            let square_count_onturn = {
                (0u8..5)
                    .filter(|square| {
                        (0u8..5)
                            .filter(|cell| self.state & 1 << square * 5 + cell != 0)
                            .count()
                            > 2
                    })
                    .count()
            };
            match square_count_onturn {
                c if c > 2 => return Some(Result::from_player(&self.onturn)),
                _ => return Some(Result::from_player(&self.onturn.other())),
            }
        }

        // The streak of consecutively taking stones is reached.
        if self.takestreak == Self::TAKESTREAK_LIMIT {
            return Some(Result::Draw);
        }

        // Check if the player onturn still has stones left.
        if self.stones[self.onturn as usize] == 0 {
            return Some(Result::from_player(&self.onturn));
        }

        return None; // The game is not over yet.
    }

    /// Return all the moves that can be made from the current position as (square, cell).
    pub fn moves(&self) -> Vec<(u8, u8)> {
        if self.isfirst() {
            vec![(0, 0), (0, 1), (0, 2), (0, 4), (2, 0), (2, 2)]
        } else {
            (0..=4)
                .map(|cell| (self.square().unwrap(), cell))
                .filter(|&(square, cell)| self.canplay(square, cell))
                .collect()
        }
    }

    /// Return how many moves have been made this game.
    pub fn movecount(&self) -> i16 {
        self.movecount
    }

    /// Return how many turns in a row pieces have been taken.
    pub fn takestreak(&self) -> u8 {
        self.takestreak
    }

    /// Return the player onturn.
    pub fn onturn(&self) -> player::Players {
        self.onturn
    }

    /// Return if this will be the first move.
    pub fn isfirst(&self) -> bool {
        self.movecount == 0
    }

    /// Return whether the provided square is full.
    pub fn isfull(&self, square: u8) -> bool {
        let mask_square = 0b11111 << square * 5;
        self.mask & mask_square == mask_square
    }

    /// Return the square constraint.
    pub fn square(&self) -> Option<u8> {
        match self.lastmove {
            None => None,
            Some((_, square)) => Some(square),
        }
    }
}

impl std::fmt::Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Get the symbol corresponding to the given cell.
        let symbol = |square: u8, cell: u8| -> String {
            let index = square * 5 + cell;
            if self.mask & 1 << index == 0 {
                ".".to_string()
            } else if self.state & 1 << index == 0 {
                self.onturn.other().to_string()
            } else {
                self.onturn.to_string()
            }
        };

        write!(
            f,
            concat!(
                "+-----------+---+-----------+\n",
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
                "+-----------+---+-----------+\n",
            ),
            symbol(0, 0),
            symbol(0, 1),
            symbol(1, 0),
            symbol(1, 1),
            symbol(0, 2),
            symbol(1, 2),
            symbol(0, 3),
            symbol(0, 4),
            symbol(1, 3),
            symbol(1, 4),
            symbol(2, 2),
            symbol(3, 0),
            symbol(3, 1),
            symbol(4, 0),
            symbol(4, 1),
            symbol(3, 2),
            symbol(4, 2),
            symbol(3, 3),
            symbol(3, 4),
            symbol(4, 3),
            symbol(4, 4)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test some basic error handling for the load function.
    #[test]
    fn load_basic() {
        assert!(Board::load("jfkd").is_err());
        assert!(Board::load("3").is_err());
        assert!(Board::load("35").is_err());
        assert!(Board::load("012345").is_err());
        assert!(Board::load("23202124220").is_err());

        assert!(Board::load("").is_ok());
        assert!(Board::load("02").is_ok());
        assert!(Board::load("01234").is_ok());
    }

    /// Test the board is correct when loaded from a string.
    #[test]
    fn load_more() {
        let board1 = Board::load("0123432100304022").unwrap();

        assert_eq!(board1.state, 0b00000_10100_00001_00101_11100);
        assert_eq!(board1.mask, 0b01001_10111_11111_01101_11111);
        assert_eq!(board1.onturn, player::Players::Player2);
        assert_eq!(board1.stones, [4, 5]);
        assert_eq!(board1.lastmove, Some((2, 2)));
        assert_eq!(board1.takestreak, 0);
        assert_eq!(board1.movecount, 15);

        let board2 = Board::load("01234321003040223").unwrap();

        assert_eq!(board2.state, 0b01001_00001_10110_01000_00011);
        assert_eq!(board2.mask, 0b01001_10111_11111_01101_11111);
        assert_eq!(board2.onturn, player::Players::Player1);
        assert_eq!(board2.stones, [5, 4]);
        assert_eq!(board2.lastmove, Some((2, 3)));
        assert_eq!(board2.takestreak, 1);
        assert_eq!(board2.movecount, 16);
    }

    /// Test state update correctness after occupying an empty cell.
    #[test]
    fn play_empty() {
        let mut board = Board::new();
        board.play(3, 4);

        assert_eq!(board.state, 0b00000_00000_00000_00000_00000);
        assert_eq!(board.mask, 0b00000_10000_00000_00000_00000);
        assert_eq!(board.onturn, player::Players::Player2);
        assert_eq!(board.stones, [11, 12]);
        assert_eq!(board.lastmove, Some((3, 4)));
        assert_eq!(board.takestreak, 0);
        assert_eq!(board.movecount, 1);

        board.play(board.square().unwrap(), 1);

        assert_eq!(board.state, 0b00000_10000_00000_00000_00000);
        assert_eq!(board.mask, 0b00010_10000_00000_00000_00000);
        assert_eq!(board.onturn, player::Players::Player1);
        assert_eq!(board.stones, [11, 11]);
        assert_eq!(board.lastmove, Some((4, 1)));
        assert_eq!(board.takestreak, 0);
        assert_eq!(board.movecount, 2);
    }

    /// Test state update correctness after taking a stone.
    #[test]
    fn play_takes() {
        let mut board = Board::load("00203010").unwrap();
        board.play(board.square().unwrap(), 0);

        assert_eq!(board.state, 0b00000_00001_00001_00001_10000);
        assert_eq!(board.mask, 0b00000_00001_00001_00001_11111);
        assert_eq!(board.stones, [9, 8]);
        assert_eq!(board.takestreak, 1);

        board.play(board.square().unwrap(), 3);

        assert_eq!(board.state, 0b00000_00000_00000_00000_00111);
        assert_eq!(board.mask, 0b00000_00001_00001_00001_11111);
        assert_eq!(board.stones, [8, 9]);
        assert_eq!(board.takestreak, 2);

        board.play(board.square().unwrap(), 4);

        assert_eq!(board.takestreak, 0);
    }

    /// Test state update correctness when playing on a double cell.
    #[test]
    fn play_double() {
        let mut board1 = Board::new();
        board1.play(0, 4);
        assert_eq!(board1.state, 0b00000_00000_00000_00000_00000);
        assert_eq!(board1.mask, 0b00000_00000_00001_00000_10000);

        let mut board2 = Board::new();
        board2.play(1, 3);
        assert_eq!(board2.mask, 0b00000_00000_00010_01000_00000);

        let mut board3 = Board::new();
        board3.play(3, 1);
        assert_eq!(board3.mask, 0b00000_00010_01000_00000_00000);

        let mut board4 = Board::new();
        board4.play(4, 0);
        assert_eq!(board4.mask, 0b00001_00000_10000_00000_00000);

        let mut board5 = Board::new();
        board5.play(2, 0);
        assert_eq!(board5.mask, 0b00000_00000_00001_00000_10000);

        let mut board6 = Board::new();
        board6.play(2, 1);
        assert_eq!(board6.mask, 0b00000_00000_00010_01000_00000);

        let mut board7 = Board::new();
        board7.play(2, 3);
        assert_eq!(board7.mask, 0b00000_00010_01000_00000_00000);

        let mut board8 = Board::new();
        board8.play(2, 4);
        assert_eq!(board8.mask, 0b00001_00000_10000_00000_00000);
    }

    /// Test playing in an empty cell.
    #[test]
    fn canplay_empty() {
        let mut board = Board::load("00").unwrap();

        assert!(!board.canplay(board.square().unwrap(), 0));

        assert!(board.canplay(board.square().unwrap(), 1));
        assert!(board.canplay(board.square().unwrap(), 2));
        assert!(board.canplay(board.square().unwrap(), 3));
        assert!(board.canplay(board.square().unwrap(), 4));

        board.play(board.square().unwrap(), 4);
        board.play(board.square().unwrap(), 0);
        board.play(board.square().unwrap(), 2);

        assert!(!board.canplay(board.square().unwrap(), 0));
        assert!(!board.canplay(board.square().unwrap(), 4));

        assert!(board.canplay(board.square().unwrap(), 1));
        assert!(board.canplay(board.square().unwrap(), 2));
        assert!(board.canplay(board.square().unwrap(), 3));

        board.play(board.square().unwrap(), 2);

        assert!(!board.canplay(board.square().unwrap(), 0));
        assert!(!board.canplay(board.square().unwrap(), 2));
        assert!(!board.canplay(board.square().unwrap(), 4));

        assert!(board.canplay(board.square().unwrap(), 1));
        assert!(board.canplay(board.square().unwrap(), 3));
    }

    /// Test taking a stone.
    #[test]
    fn canplay_takes_normal() {
        let mut board1 = Board::load("00203010").unwrap();

        assert!(board1.canplay(board1.square().unwrap(), 0));
        assert!(board1.canplay(board1.square().unwrap(), 4));

        assert!(!board1.canplay(board1.square().unwrap(), 1));
        assert!(!board1.canplay(board1.square().unwrap(), 2));
        assert!(!board1.canplay(board1.square().unwrap(), 3));

        board1.play(board1.square().unwrap(), 0);

        assert!(board1.canplay(board1.square().unwrap(), 1));
        assert!(board1.canplay(board1.square().unwrap(), 2));
        assert!(board1.canplay(board1.square().unwrap(), 3));

        assert!(!board1.canplay(board1.square().unwrap(), 0));
        assert!(!board1.canplay(board1.square().unwrap(), 4));

        board1.play(board1.square().unwrap(), 2);

        assert!(board1.canplay(board1.square().unwrap(), 1));
        assert!(board1.canplay(board1.square().unwrap(), 2));
        assert!(board1.canplay(board1.square().unwrap(), 3));
        assert!(board1.canplay(board1.square().unwrap(), 4));

        assert!(!board1.canplay(board1.square().unwrap(), 0));

        let board2 = Board::load("11210141").unwrap();

        assert!(board2.canplay(board2.square().unwrap(), 1));
        assert!(board2.canplay(board2.square().unwrap(), 3));

        assert!(!board2.canplay(board2.square().unwrap(), 0));
        assert!(!board2.canplay(board2.square().unwrap(), 2));
        assert!(!board2.canplay(board2.square().unwrap(), 4));
    }

    /// Test taking the previous move.
    #[test]
    fn canplay_takes_prev() {
        // normal cannot take
        let board1 = Board::load("12101411").unwrap();
        assert!(!board1.canplay(board1.square().unwrap(), 1));

        // double cannot take
        let board2 = Board::load("442343214122024").unwrap();
        assert!(board2.canplay(board2.square().unwrap(), 2));
        assert!(!board2.canplay(board2.square().unwrap(), 0));

        // we can take
        let board3 = Board::load("24232021122").unwrap();
        assert!(board3.canplay(board3.square().unwrap(), 2));
    }

    /// Test winning by completing a square.
    #[test]
    fn isover_square() {
        let board1 = Board::load("2320212422").unwrap();
        assert_eq!(board1.isover(), Some(Result::Player1));

        let board2 = Board::load("22021232422").unwrap();
        assert_eq!(board2.isover(), Some(Result::Player2));
    }

    /// Test winning on a full board.
    #[test]
    fn isover_full() {
        let board1 = Board::load("200301314022334323344241120010").unwrap();
        assert_eq!(board1.isover(), Some(Result::Player2));

        let board2 = Board::load("2003310221243201141030223442").unwrap();
        assert_eq!(board2.isover(), Some(Result::Player1));
    }

    /// Test drawing because the takestreak is reached.
    #[test]
    fn isover_takestreak() {
        if Board::TAKESTREAK_LIMIT < 5 || Board::TAKESTREAK_LIMIT > 30 {
            panic!("Please keep the TAKESTREAK_LIMIT between 5 and 30.");
        }

        // Setup the start of the game, after this we can cycle with 21103.
        // The takestreak is already 2 here.
        let start = String::from("20033102212432011410302234201");
        let cycle = "21103".repeat(6);
        let cycle = cycle.get(..Board::TAKESTREAK_LIMIT as usize - 2).unwrap();

        let board = Board::load(&(start + cycle)).unwrap();
        assert_eq!(board.isover(), Some(Result::Draw));
    }

    /// Test winning because the player onturn has no stones left.
    #[test]
    fn isover_stones() {
        let board = Board::load("0020301101440313322423412").unwrap();
        assert_eq!(board.isover(), Some(Result::Player1));
    }

    /// Test winning in the fastest possible way.
    #[test]
    fn isover_movecount() {
        let board = Board::load("2021232422").unwrap();
        assert_eq!(board.isover(), Some(Result::Player1));
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
                "+-----------+---+-----------+\n",
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
                "+-----------+---+-----------+\n",
            )
        );
    }

    /// Test display of empty board.
    #[test]
    fn display_empty() {
        assert_eq!(
            format!("{}", Board::new()),
            concat!(
                "+-----------+---+-----------+\n",
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
                "+-----------+---+-----------+\n",
            )
        );
    }

    #[test]
    fn clone() {
        let board = Board::load("0020").unwrap();
        let mut board_clone = board.clone();

        assert_eq!(board.state, board_clone.state);
        assert_eq!(board.mask, board_clone.mask);
        assert_eq!(board.onturn, board_clone.onturn);
        assert_eq!(board.stones[0], board_clone.stones[0]);
        assert_eq!(board.stones[1], board_clone.stones[1]);
        assert_eq!(board.lastmove, board_clone.lastmove);
        assert_eq!(board.takestreak, board_clone.takestreak);
        assert_eq!(board.movecount, board_clone.movecount);

        board_clone.play(0, 1);
        board_clone.play(1, 4);

        assert_ne!(board.stones[0], board_clone.stones[0]);
        assert_ne!(board.stones[1], board_clone.stones[1]);
        assert_ne!(board.lastmove, board_clone.lastmove);
    }
}
