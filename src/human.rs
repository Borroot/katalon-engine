use crate::{board, player};
use regex;
use std::io::{self, Write};

/// A player controlled by a puny human.
pub struct Human;

impl player::Player for Human {
    fn play(&self, board: &board::Board) -> (u8, u8) {
        loop {
            // Read the input
            print!("{} > ", board.onturn());
            io::stdout().flush().unwrap();

            let mut line = String::new();
            io::stdin().read_line(&mut line).unwrap();

            let numbers = line.trim();
            let re = regex::Regex::new(r"^(?P<square>[0-4]?)(?P<cell>[0-4])$").unwrap();

            if !re.is_match(numbers) {
                println!("Please use the move format: [0-4]<0-4>.");
                continue;
            }

            // Extract the input
            let caps = re.captures(numbers).unwrap();

            let cell = caps.name("cell").unwrap().as_str().chars().next().unwrap();
            let cell = cell as u8 - '0' as u8;

            let square;

            if let Some(s) = caps.name("square").unwrap().as_str().chars().next() {
                square = s as u8 - '0' as u8;

                // Make sure the correct square is provided.
                if !board.isfirst() && square != board.square().unwrap() {
                    println!(
                        "{}",
                        format!(
                            concat!(
                                "Error: you provided the square {}, ",
                                "but the square constraint is on {}.\n",
                                "Hint: you don't have to specify the square.",
                            ),
                            square,
                            board.square().unwrap()
                        )
                    );
                    continue;
                }
            } else {
                if board.isfirst() {
                    println!("Error: please also provide the square.");
                    continue;
                }
                square = board.square().unwrap()
            }

            // Check and return the input
            if !board.canplay(square, cell) {
                println!("Error: illegal move.");
                continue;
            }
            return (square, cell);
        }
    }
}
