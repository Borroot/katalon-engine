use crate::player;

impl super::Board {
    /// All the mappings for symmetrical boards.
    #[rustfmt::skip]
    const SYMMETRIES: [[usize; 25]; 7] = [
        // Flip diagonal 1-3:
        [24,21,22,23,20,9,6,7,8,5,14,11,12,13,10,19,16,17,18,15,4,1,2,3,0],
        // Flip diagonal 0-4:
        [0,3,2,1,4,15,18,17,16,19,10,13,12,11,14,5,8,7,6,9,20,23,22,21,24],
        // Flip horizontal:
        [18,19,17,15,16,23,24,22,20,21,13,14,12,10,11,3,4,2,0,1,8,9,7,5,6],
        // Flip vertical:
        [6,5,7,9,8,1,0,2,4,3,11,10,12,14,13,21,20,22,24,23,16,15,17,19,18],
        // Rotation 90:
        [18,15,17,19,16,3,0,2,4,1,13,10,12,14,11,23,20,22,24,21,8,5,7,9,6],
        // Rotation 180:
        [24,23,22,21,20,19,18,17,16,15,14,13,12,11,10,9,8,7,6,5,4,3,2,1,0],
        // Rotation 270:
        [6,9,7,5,8,21,24,22,20,23,11,14,12,10,13,1,4,2,0,3,16,19,17,15,18],
    ];

    /// Indiciate whether the square of the lastmove should be in the key.
    /// This is the case if the square we need to move into next is full.
    fn lastmove_square(&self) -> bool {
        let (_, cell) = self.lastmove.unwrap();
        self.isfull(cell)
    }

    /// Map the state or mask to the given symmetry.
    fn symmetry_map(value: u32, symmetry: &[usize; 25]) -> u64 {
        let mut symmetry_value: u32 = 0;
        for index in 0..25 {
            let bit = (value & 1 << index) >> index;
            symmetry_value += bit << symmetry[index];
        }
        symmetry_value as u64
    }

    /// Return a u64 uniquely identifying this state of the board.
    pub fn key(&self) -> u64 {
        // 1 bit onturn + 7 bits takestreak + 6 bits lastmove + 25 bits mask + 25 bits state
        let mut key: u64 = 0;

        // The board is completely empty (so lastmove == None).
        if self.isfirst() {
            return key;
        }

        // Add the player onturn and the takestreak.
        key += (self.onturn as u64) << 63;
        key += (self.takestreak as u64) << 56;

        // Add the lastmove to the key.
        key += (self.lastmove.unwrap().1 as u64) << 50; // add the cell
        if self.lastmove_square() {
            key += (self.lastmove.unwrap().0 as u64) << 53; // add the square
        }

        // Add the mask and the state.
        key += (self.mask as u64) << 25;
        key += self.state as u64;

        key
    }

    /// Return all u64 uniquely identifying this equivalence class of the board.
    // TODO convert this to an iterator implementation
    pub fn keys(&self) -> [u64; 8] {
        let mut keys: [u64; 8] = [0; 8];

        // The board is completely empty (so lastmove == None).
        if self.isfirst() {
            return keys;
        }

        keys[0] = self.key();
        let lastmove_square = self.lastmove_square();

        // Generate the keys for all the symmetries.
        for index in 0..7 {
            // Add the player onturn and the takestreak.
            keys[index + 1] += (self.onturn as u64) << 63;
            keys[index + 1] += (self.takestreak as u64) << 56;

            // Add the lastmove to the key. Create the lastmove symmetry.
            let lastmove_index = self.lastmove.unwrap().0 * 5 + self.lastmove.unwrap().1;
            let lastmove_symmetry = (
                Self::SYMMETRIES[index][lastmove_index as usize] / 5,
                Self::SYMMETRIES[index][lastmove_index as usize] % 5,
            );

            // Add the cell and the square to the key.
            keys[index + 1] += (lastmove_symmetry.1 as u64) << 50; // cell
            if lastmove_square {
                keys[index + 1] += (lastmove_symmetry.0 as u64) << 53; // square
            }

            // Add the mask and the state.
            keys[index + 1] += Self::symmetry_map(self.mask, &Self::SYMMETRIES[index]) << 25;
            keys[index + 1] += Self::symmetry_map(self.state, &Self::SYMMETRIES[index]);
        }

        keys
    }

    /// Create a board from a key. Note that no checks on correctness are made.
    pub fn from_key(mut key: u64, movecount: i16) -> Self {
        let mut board = Self::new();

        // Return early if the board is empty (this is neccessary because of lastmove).
        if key == 0 {
            return board;
        }

        // Read all the bits from right to left (LSB to MSB).
        // 1 bit onturn + 7 bits takestreak + 6 bits lastmove + 25 bits mask + 25 bits state

        // Load the state.
        board.state = (key & u64::pow(2, 25) - 1) as u32;
        key >>= 25;

        // Load the mask.
        board.mask = (key & u64::pow(2, 25) - 1) as u32;
        key >>= 25;

        // Load the lastmove.
        let cell = (key & u64::pow(2, 3) - 1) as u8;
        key >>= 3;
        let square = (key & u64::pow(2, 3) - 1) as u8;
        key >>= 3;
        board.lastmove = Some((square, cell));

        // Load the takestreak.
        board.takestreak = (key & u64::pow(2, 7) - 1) as u8;
        key >>= 7;

        // Load the onturn.
        board.onturn = player::Players::from_index(key as usize).expect("Onturn should be 0 or 1.");

        // Set the movecount.
        board.movecount = movecount;

        // Count the number of stones a player has left, by counting the number of stones placed.
        let count_stones = |state: u32| {
            let mut count = state.count_ones() as u8;
            // Stones at the double positions are counted double...
            let doubles = [
                0b0000000000100000000000000,
                0b0000000000010000000000000,
                0b0000000000000100000000000,
                0b0000000000000010000000000,
            ];
            for double in doubles {
                if state & double > 0 {
                    count -= 1;
                }
            }
            Self::NUMBER_OF_STONES - count
        };

        // Deduce the number of stones each player has left.
        board.stones[board.onturn as usize] = count_stones(board.state);
        board.stones[1 ^ board.onturn as usize] = count_stones(board.state ^ board.mask);

        board
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;

    /// Test whether the lastmove square is included properly.
    #[test]
    fn lastmove_square() {
        let board1 = Board::load("221400203101122").unwrap();
        assert!(board1.lastmove_square());

        let board2 = Board::load("221400203101123").unwrap();
        assert!(!board2.lastmove_square());

        let board3 = Board::load("2214002031011232").unwrap();
        assert!(board3.lastmove_square());
    }

    /// Test symmetry mapping.
    #[test]
    fn symmetry_map() {
        let v1: u32 = 0b00001_00010_11010_01011_01100;
        let s1: u64 = 0b00110_11010_01011_01000_10000;
        assert_eq!(s1, Board::symmetry_map(v1, &Board::SYMMETRIES[5]))
    }

    /// Test first move key generation.
    #[test]
    fn key_firstmove() {
        let board = Board::new();
        assert_eq!(board.key(), 0);
    }

    /// Test single key generation.
    #[test]
    fn key() {
        let board1 = Board::load("221400203101122").unwrap();
        assert_eq!(
            board1.key(),
            0b00000001__010_010__00001_00010_11111_11111_11111__00000_00010_01001_10100_10011
        );

        let board2 = Board::load("221400203101123").unwrap();
        assert_eq!(
            board2.key(),
            0b00000001__000_011__00001_00010_11111_11111_11111__00000_00000_00101_10100_10011
        );

        let board3 = Board::load("2214002031011232").unwrap();
        assert_eq!(
            board3.key(),
            0b10000000__011_010__00001_00110_11111_11111_11111__00001_00010_11010_01011_01100
        );
    }

    /// Test all symmetry key generation.
    #[test]
    fn keys() {
        let board1 = Board::load("221400203101122").unwrap();
        let keys1 = board1.keys();

        assert_eq!(
            keys1[0],
            0b00000001__010_010__00001_00010_11111_11111_11111__00000_00010_01001_10100_10011
        );

        assert_eq!(
            keys1[6],
            0b00000001__010_010__11111_11111_11111_01000_10000__11001_00101_10010_01000_00000
        );

        let board2 = Board::load("221400203101123").unwrap();
        let keys2 = board2.keys();

        assert_eq!(
            keys2[0],
            0b00000001__000_011__00001_00010_11111_11111_11111__00000_00000_00101_10100_10011
        );

        assert_eq!(
            keys2[6],
            0b00000001__000_001__11111_11111_11111_01000_10000__11001_00101_10100_00000_00000
        );

        let board3 = Board::load("2214002031011232").unwrap();
        let keys3 = board3.keys();

        assert_eq!(
            keys3[0],
            0b10000000__011_010__00001_00110_11111_11111_11111__00001_00010_11010_01011_01100
        );

        assert_eq!(
            keys3[6],
            0b10000000__001_010__11111_11111_11111_01100_10000__00110_11010_01011_01000_10000
        );
    }

    /// Test first move symmetry key generation.
    #[test]
    fn keys_firstmove() {
        let board1 = Board::new();
        for key in board1.keys() {
            assert_eq!(key, 0);
        }

        let board2 = Board::load("21").unwrap();
        let keys2 = board2.keys();

        assert_eq!(
            keys2[0],
            0b10000000__000_001__00000_00000_00010_01000_00000__00000_00000_00000_00000_00000
        );

        assert_eq!(
            keys2[4],
            0b10000000__000_000__00000_00000_00001_00000_10000__00000_00000_00000_00000_00000
        );
    }

    /// Test creation of a board from the zero key.
    #[test]
    fn from_key_zero() {
        let board = Board::from_key(0, 0);
        assert_eq!(board.key(), 0);
    }

    /// Test creation of a board from a key.
    #[test]
    fn from_key_many() {
        for _ in 0..10000 {
            let board_orig = Board::random();
            let board_copy = Board::from_key(board_orig.key(), board_orig.movecount);

            assert_eq!(board_orig.key(), board_copy.key());

            assert_eq!(
                board_orig.stones[0], board_copy.stones[0],
                "player X stone count"
            );
            assert_eq!(
                board_orig.stones[1], board_copy.stones[1],
                "player O stone count"
            );
        }
    }
}
