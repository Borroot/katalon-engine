use crate::{board, player};

type Player = Box<dyn player::Player>;

pub struct Game {
    players: [Player; 2],
    board: board::Board,
    verbose: bool,
}

impl Game {
    pub fn new(player1: Player, player2: Player, verbose: bool) -> Self {
        Game {
            players: [player1, player2],
            board: board::Board::new(),
            verbose,
        }
    }

    /// Play the game and return the result of the round.
    pub fn run(&mut self) -> board::Result {
        if self.board.isover() != None {
            panic!("Game already finished.");
        }

        if self.verbose {
            print!("{}", self.board);
        }

        while self.board.isover() == None {
            let player = &self.players[self.board.onturn() as usize];
            let (square, cell) = player.play(&self.board);
            self.board.play(square, cell);

            if self.verbose {
                print!("\n{}", self.board);
            }
        }
        self.board.isover().unwrap()
    }
}
