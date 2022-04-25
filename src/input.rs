use crate::board;
use std::io::Write;

/// Return the regex used to recognize a move of the form [0-4]<0-4>.
pub fn move_regex() -> regex::Regex {
    regex::Regex::new(r"^(?P<square>[0-4]?)(?P<cell>[0-4])$").unwrap()
}

/// Request and return user input.
pub fn request(prompt: String) -> String {
    print!("{}", prompt);
    std::io::stdout().flush().unwrap();

    let mut line = String::new();
    std::io::stdin().read_line(&mut line).unwrap();

    line.trim().to_string()
}

/// Extract the square from the given board and optionally given square.
fn extract_square(board: &board::Board, square: Option<u8>) -> Result<u8, String> {
    if let Some(square) = square {
        // Make sure the correct square is provided.
        if !board.isfirst() && square != board.square().unwrap() {
            return Err(format!(
                concat!(
                    "Error: square should be {}, not {}.\n",
                    "Hint: you don't have to specify the square.",
                ),
                board.square().unwrap(),
                square
            ));
        }
        return Ok(square);
    } else {
        if board.isfirst() {
            return Err(String::from("Error: please also provide the square."));
        }
        return Ok(board.square().unwrap());
    }
}

/// Extract the square and cell from the given string.
pub fn extract(board: &board::Board, text: &str) -> Result<(u8, u8), String> {
    debug_assert!(move_regex().is_match(text));

    let re = move_regex();
    let caps = re.captures(text).unwrap();

    let extract_number = |key: &str| match caps.name(key).unwrap().as_str().chars().next() {
        Some(n) => Some(n as u8 - '0' as u8),
        None => None,
    };

    let cell = extract_number("cell").unwrap();
    let square = extract_square(board, extract_number("square"))?;

    // Check and return the input
    if !board.canplay(square, cell) {
        return Err(String::from("Error: illegal move."));
    }
    return Ok((square, cell));
}
