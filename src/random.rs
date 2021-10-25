use crate::{board, player};
use rand::prelude::*;

/// A player wich always makes a random move.
pub struct Random;

impl player::Player for Random {
    fn play(&self, board: &board::Board) -> (u8, u8) {
        let mut rng = rand::thread_rng();

        if board.isfirst() {
            let square: u8 = rng.gen_range(0..=4);
            let cell: u8 = rng.gen_range(0..=4);
            return (square, cell);
        } else {
            let square = board.square().unwrap();
            let cell = vec![0, 1, 2, 3, 4];
            let options: Vec<_> = cell
                .iter()
                .filter(|&&cell| board.canplay(square, cell))
                .collect();
            let cell = *options[rng.gen_range(0..options.len()) as usize];
            return (square, cell);
        }
    }
}
