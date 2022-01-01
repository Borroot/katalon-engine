use katalon::{board, game};

#[allow(unused_imports)]
use katalon::{human, random, solver};

fn main() {
    loop {
        let player1 = Box::new(random::Random);
        let player2 = Box::new(solver::Solver);

        let mut game = game::Game::new(player1, player2, false);
        let result = game.run();
        match result {
            board::Result::Draw => println!("It's a draw!"),
            _ => println!("Player {} won!", result.player().unwrap()),
        }
    }
}
