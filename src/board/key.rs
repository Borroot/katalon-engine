impl super::Board {
    /// All the mappings for symmetrical boards.
    #[rustfmt::skip]
    const SYMMETRIES: [[usize; 25]; 7] = [
        // Flip diagonal 1-3:
        [24,21,22,23,20,9,6,7,8,5,20,8,12,16,4,19,16,17,18,15,4,1,2,3,0],
        // Flip diagonal 0-4:
        [0,3,2,1,4,15,18,17,16,19,4,16,12,8,20,5,8,7,6,9,20,23,22,21,24],
        // Flip horizontal:
        [18,19,17,15,16,23,24,22,20,21,16,20,12,4,8,3,4,2,0,1,8,9,7,5,6],
        // Flip vertical:
        [6,5,7,9,8,1,0,2,4,3,8,4,12,20,16,21,20,22,24,23,16,15,17,19,18],
        // Rotation 90:
        [18,15,17,19,16,3,0,2,4,1,16,4,12,20,8,23,20,22,24,21,8,5,7,9,6],
        // Rotation 180:
        [24,23,22,21,20,19,18,17,16,15,14,13,12,11,10,9,8,7,6,5,4,3,2,1,0],
        // Rotation 270:
        [6,9,7,5,8,21,24,22,20,23,8,20,12,4,16,1,4,2,0,3,16,19,17,15,18],
    ];

    /// Indiciate whether the square of the lastmove should be in the key.
    fn lastmove_square(&self) -> bool {
        // The 6 bytes of lastmove will only contain the square if that square
        // is full and (square == cell or double(square, cell) != None).
        if self.lastmove != None {
            let mask_square = 0b11111 << self.lastmove.unwrap().0 * 5;
            let (square, cell) = self.lastmove.unwrap();
            return self.mask & mask_square == mask_square
                && (square == cell || Self::double(square, cell) != None);
        }
        false
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
        // 4 bytes takestreak + 6 bytes lastmove + 1 byte onturn + 25 bytes mask + 25 bytes state
        let mut key: u64 = 0;

        // Take the first four bytes of the takestreak.
        key += (self.takestreak as u64) << 57;

        // Add the lastmove to the key.
        if self.lastmove != None {
            key += (self.lastmove.unwrap().1 as u64) << 51; // add the cell
            if self.lastmove_square() {
                key += (self.lastmove.unwrap().0 as u64) << 54; // add the square
            }
        }

        // Add the player onturn: 0 if onturn == player1 else 1.
        key += (self.onturn as u64) << 50;

        // Add the mask and the state.
        key += (self.mask as u64) << 25;
        key += self.state as u64;

        key
    }

    /// Return all u64 uniquely identifying this equivalence class of the board.
    pub fn keys(&self) -> [u64; 8] {
        let mut keys: [u64; 8] = [0; 8];
        keys[0] = self.key();

        let lastmove_square = self.lastmove_square();

        // Generate the keys for all the symmetries.
        for index in 0..7 {
            // Take the first four bytes of the takestreak.
            keys[index + 1] += (self.takestreak as u64) << 57;

            // Add the lastmove to the key.
            if self.lastmove != None {
                // Create the lastmove symmetry.
                let lastmove_index = self.lastmove.unwrap().0 * 5 + self.lastmove.unwrap().1;
                let lastmove_symmetry = (
                    Self::SYMMETRIES[index][lastmove_index as usize] / 5,
                    Self::SYMMETRIES[index][lastmove_index as usize] % 5,
                );

                // Add the cell and the square to the key.
                keys[index + 1] += (lastmove_symmetry.1 as u64) << 51; // cell
                if lastmove_square {
                    keys[index + 1] += (lastmove_symmetry.0 as u64) << 54;
                }
            }

            // Add the player onturn: 0 if onturn == player1 else 1.
            keys[index + 1] += (self.onturn as u64) << 50;

            // Add the mask and the state.
            keys[index + 1] += Self::symmetry_map(self.mask, &Self::SYMMETRIES[index]) << 25;
            keys[index + 1] += Self::symmetry_map(self.state, &Self::SYMMETRIES[index]);
        }

        keys
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
        assert!(board2.lastmove_square());

        let board3 = Board::load("2214002031011232").unwrap();
        assert!(!board3.lastmove_square());
    }

    /// Test symmetry mapping.
    #[test]
    fn symmetry_map() {
        let v1: u32 = 0b00001_00010_11010_01011_01100;
        let s1: u64 = 0b00110_11010_01011_01000_10000;
        assert_eq!(s1, Board::symmetry_map(v1, &Board::SYMMETRIES[5]))
    }

    /// Test single key generation.
    #[test]
    fn key() {
        let board1 = Board::load("221400203101122").unwrap();
        assert_eq!(
            board1.key(),
            0b000__0001__010_010__0__00001_00010_11111_11111_11111__00000_00010_01001_10100_10011
        );

        let board2 = Board::load("221400203101123").unwrap();
        assert_eq!(
            board2.key(),
            0b000__0001__010_011__0__00001_00010_11111_11111_11111__00000_00000_00101_10100_10011
        );

        let board3 = Board::load("2214002031011232").unwrap();
        assert_eq!(
            board3.key(),
            0b000__0000__000_010__1__00001_00110_11111_11111_11111__00001_00010_11010_01011_01100
        );
    }

    /// Test all symmetry key generation.
    #[test]
    fn keys() {
        let board1 = Board::load("221400203101122").unwrap();
        let keys1 = board1.keys();

        assert_eq!(
            keys1[0],
            0b000__0001__010_010__0__00001_00010_11111_11111_11111__00000_00010_01001_10100_10011
        );

        assert_eq!(
            keys1[6],
            0b000__0001__010_010__0__11111_11111_11111_01000_10000__11001_00101_10010_01000_00000
        );

        let board2 = Board::load("221400203101123").unwrap();
        let keys2 = board2.keys();

        assert_eq!(
            keys2[0],
            0b000__0001__010_011__0__00001_00010_11111_11111_11111__00000_00000_00101_10100_10011
        );

        assert_eq!(
            keys2[6],
            0b000__0001__010_001__0__11111_11111_11111_01000_10000__11001_00101_10100_00000_00000
        );

        let board3 = Board::load("2214002031011232").unwrap();
        let keys3 = board3.keys();
        assert_eq!(
            keys3[6],
            0b000__0000__000_010__1__11111_11111_11111_01100_10000__00110_11010_01011_01000_10000
        );
    }
}
