use katalon::player::Player;
use katalon::{board, eval, random, solver};
use std::time;

fn generate(depth: usize) -> (board::Board, String) {
    let player = random::Random;
    let mut board;
    let mut notation;

    loop {
        let mut depth_reached = true;

        board = board::Board::new();
        notation = String::new();

        for _ in 0..depth {
            // Get the move from the player.
            let (square, cell) = player.play(&board);

            // Update the notation.
            if board.isfirst() {
                notation.push_str(&square.to_string());
            }
            notation.push_str(&cell.to_string());

            // Make the move.
            board.play(square, cell);

            // Early exit if the game is over.
            if board.isover() != None {
                depth_reached = false;
                break;
            }
        }

        if depth_reached {
            return (board, notation);
        }
    }
}

fn evaluate(board: &board::Board, timeout: &time::Duration) -> Result<(eval::Eval, u128), ()> {
    let now = time::Instant::now();

    match solver::bestmoves_timeout(&board, *timeout) {
        Ok((eval, _)) => {
            return Ok((eval, now.elapsed().as_millis()));
        }
        Err(_) => return Err(()),
    }
}

/// Used to generate boards with results for the benchmarker.
fn main() {
    let timeout = time::Duration::from_secs(5);
    let depth = 15;

    loop {
        let (board, notation) = generate(depth);
        if let Ok((eval, time)) = evaluate(&board, &timeout) {
            println!("{} {} {}", notation, eval, time);
        }
    }
}
