use katalon::board;
use std::collections::HashSet;

// TODO evaluate and save all openings for quick lookup later

fn main() {
    let mut states = HashSet::<u64>::new();
    let depth: usize = 2;

    backtrack(board::Board::new(), depth, &mut states);
    println!("number of states until depth {} is {}", depth, states.len());
}

fn backtrack(board: board::Board, depth: usize, states: &mut HashSet<u64>) {
    if depth == 0 {
        return;
    }

    let mut recurs = |square: u8, cell: u8| {
        if board.canplay(square, cell) {
            let mut board_clone = board.clone();
            board_clone.play(square, cell);

            for key in board_clone.keys() {
                if states.contains(&key) {
                    return;
                }
            }

            states.insert(board_clone.key());
            backtrack(board_clone, depth - 1, states);
        }
    };

    if board.isfirst() {
        for square in 0..5 {
            for cell in 0..5 {
                recurs(square, cell);
            }
        }
    } else {
        for cell in 0..5 {
            recurs(board.square().unwrap(), cell);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn depth1() {
        let mut states = HashSet::<u64>::new();
        let depth: usize = 1;

        backtrack(board::Board::new(), depth, &mut states);
        assert_eq!(6, states.len());
    }

    #[test]
    fn depth2() {
        let mut states = HashSet::<u64>::new();
        let depth: usize = 2;

        backtrack(board::Board::new(), depth, &mut states);
        assert_eq!(26, states.len());
    }
}