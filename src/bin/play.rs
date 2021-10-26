use katalon::{board, game, player};

#[allow(unused_imports)]
use katalon::{human, minmax, random};

fn main() {
    let player1 = Box::new(human::Human);
    let player2 = Box::new(human::Human);

    let mut game = game::Game::new(player1, player2, true);
    let result = game.run();
    match result {
        board::Result::Draw => println!("It's a draw!"),
        _ => println!("Player {} won!", result.player().unwrap()),
    }
}
