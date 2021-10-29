use crate::{board, player, input};

/// A player controlled by a puny human.
pub struct Human;

impl player::Player for Human {
    fn play(&self, board: &board::Board) -> (u8, u8) {
        loop {
            let line = input::request(format!("{} > ", board.onturn()));
            let re = input::move_regex();

            if !re.is_match(&line) {
                println!("Please use the move format: [0-4]<0-4>.");
                continue;
            }

            match input::extract(board, &line) {
                Ok((square, cell)) => return (square, cell),
                Err(e) => {
                    println!("{}", e);
                    continue;
                }
            }
        }
    }
}
