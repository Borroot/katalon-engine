use katalon::{board, game, player};

#[allow(unused_imports)]
use katalon::{human, random};

fn main() {
    let player1 = Box::new(human::Human);
    let player2 = Box::new(random::Random);

    let mut game = game::Game::new(player1, player2, true);
    match game.run() {
        board::Result::Player1 => println!("Player {} won!", player::Players::Player1),
        board::Result::Player2 => println!("Player {} won!", player::Players::Player2),
        board::Result::Draw => println!("It's a draw!"),
    }
}
