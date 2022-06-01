use katalon::board;
use std::collections::HashSet;

fn main() {
    for depth in 0..=15 {
        stats(depth);
    }
}

/// Go through all the unique states of the board till the given depth and add
/// them to the hashset.
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

/// Simply go through all the board states till the given depth.
fn backtrack_count(board: board::Board, depth: usize, count: &mut usize) {
    if depth == 0 {
        return;
    }

    let mut recurs = |square: u8, cell: u8| {
        if board.canplay(square, cell) {
            let mut board_clone = board.clone();
            board_clone.play(square, cell);

            *count += 1;
            backtrack_count(board_clone, depth - 1, count);
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

/// Print stats: unique count, all count and unique / all.
fn stats(depth: usize) {
    let mut states = HashSet::<u64>::new();
    backtrack(board::Board::new(), depth, &mut states);
    let count_unique = states.len();

    let mut count_all = 0;
    backtrack_count(board::Board::new(), depth, &mut count_all);

    println!("max movecount = {}", depth);
    println!("count unique  = {}", count_unique);
    println!("count all     = {}", count_all);
    println!("unique / all  = {:.4}", count_unique as f64 / count_all as f64);
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
