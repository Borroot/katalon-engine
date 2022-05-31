use katalon::{board, eval, player::Player, random, solver};

/// Generate a board and its notation with movecount equal to given depth.
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

/// Try to evaluate the board within the given timelimit.
fn evaluate(board: &board::Board, timeout: &std::time::Duration) -> Result<(eval::Eval, u128), ()> {
    let now = std::time::Instant::now();

    match solver::bestmoves(&board, *timeout) {
        Ok((eval, _)) => {
            return Ok((eval, now.elapsed().as_millis()));
        }
        Err(_) => return Err(()),
    }
}

/// Used to generate boards with results for benchmarking.
fn main() {
    let timeout = std::time::Duration::from_secs(10);
    let depth = 10;

    loop {
        let (board, notation) = generate(depth);
        if let Ok((eval, time)) = evaluate(&board, &timeout) {
            println!("{}, {}, {}ms", notation, eval, time);
        } else {
            println!("{}, timeout", notation);
        }
    }
}
