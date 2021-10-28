use cmd::Cmd;
use katalon::{board, minmax, player};
use rand::prelude::*;
use regex;
use std::io::{self, Write};
use std::{fmt, time};

pub struct State {
    board: board::Board,
    notation: String,
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.board.isfirst() {
            write!(f, "{}", self.board)
        } else {
            write!(f, "{}= {}\n", self.board, self.notation)
        }
    }
}

mod cmd {
    use super::*;

    /// Signature of command function pointers.
    pub type Cmd = fn(&mut State, &[&str]) -> bool;

    pub fn play_regex() -> regex::Regex {
        regex::Regex::new(r"^(?P<square>[0-4]?)(?P<cell>[0-4])$").unwrap()
    }

    pub fn play(state: &mut State, args: &[&str]) -> bool {
        if state.board.isover() != None {
            println!("Warn: the game already finished.");
            return false;
        }

        let caps = cmd::play_regex().captures(args[0]).unwrap();

        let cell = caps.name("cell").unwrap().as_str().chars().next().unwrap();
        let cell = cell as u8 - '0' as u8;

        let square;

        if let Some(s) = caps.name("square").unwrap().as_str().chars().next() {
            square = s as u8 - '0' as u8;

            // Make sure the correct square is provided.
            if !state.board.isfirst() && square != state.board.square().unwrap() {
                println!(
                    "{}",
                    format!(
                        concat!(
                            "Error: square should be {}, not {}.\n",
                            "Hint: you don't have to specify the square.",
                        ),
                        state.board.square().unwrap(),
                        square
                    )
                );
                return false;
            }
        } else {
            if state.board.isfirst() {
                println!("Error: please also provide the square.");
                return false;
            }
            square = state.board.square().unwrap()
        }

        if !state.board.canplay(square, cell) {
            println!("Error: illegal move.");
            return false;
        }

        // Update the board and notation
        if state.board.isfirst() {
            state.notation.push_str(&square.to_string());
        }
        state.notation.push_str(&cell.to_string());
        state.board.play(square, cell);

        print!("{}", state);

        match state.board.isover() {
            Some(board::Result::Player1) => println!("Player {} won!", player::Players::Player1),
            Some(board::Result::Player2) => println!("Player {} won!", player::Players::Player2),
            Some(board::Result::Draw) => println!("It's a draw!"),
            None => (),
        }
        false
    }

    pub fn undo(state: &mut State, _args: &[&str]) -> bool {
        match state.board.movecount() {
            0 => return false,
            1 => {
                state.notation.clear();
                state.board = board::Board::new();
            }
            _ => {
                state.notation.pop();
                state.board = board::Board::load(&state.notation).unwrap();
            }
        }
        print!("{}", state);
        false
    }

    pub fn eval(state: &mut State, args: &[&str]) -> bool {
        if state.board.isover() != None {
            println!("Warn: the game already finished.");
            return false;
        }

        let mut timeout = Some(10);

        if args.len() > 1 {
            if let Ok(time) = args[1].parse::<u64>() {
                timeout = Some(time);
            } else {
                println!("Warn: invalid timeout ignored");
            }
        }

        let now = time::Instant::now();

        let result = {
            if timeout == None {
                Ok(minmax::Minmax::bestmoves(&state.board))
            } else {
                minmax::Minmax::bestmoves_timeout(
                    &state.board,
                    time::Duration::from_secs(timeout.unwrap()),
                )
            }
        };

        if result.is_err() {
            println!("timeout after {}s", timeout.unwrap());
            return false;
        }

        let (mut value, bestmoves) = result.unwrap();
        value = minmax::Minmax::humanize_relative(state.board.movecount() as isize, value);

        println!(
            "evaluation: {} in {}ms\nmoves: {:?}",
            value,
            now.elapsed().as_millis(),
            bestmoves
        );

        false
    }

    pub fn best(state: &mut State, args: &[&str]) -> bool {
        if state.board.isover() != None {
            println!("Warn: the game already finished.");
            return false;
        }

        let mut timeout = Some(10);

        if args.len() > 1 {
            if let Ok(time) = args[1].parse::<u64>() {
                timeout = Some(time);
            } else {
                println!("Warn: invalid timeout ignored");
            }
        }

        let now = time::Instant::now();

        let result = {
            if timeout == None {
                Ok(minmax::Minmax::bestmoves(&state.board))
            } else {
                minmax::Minmax::bestmoves_timeout(
                    &state.board,
                    time::Duration::from_secs(timeout.unwrap()),
                )
            }
        };

        if result.is_err() {
            println!("timeout after {}s", timeout.unwrap());
            return false;
        }

        let (mut value, bestmoves) = result.unwrap();
        value = minmax::Minmax::humanize_relative(state.board.movecount() as isize, value);

        let mut rng = rand::thread_rng();
        let bestmove = bestmoves[rng.gen_range(0..bestmoves.len()) as usize];

        println!(
            "evaluation: {} in {}ms\n{:?} -> {:?}",
            value,
            now.elapsed().as_millis(),
            bestmoves,
            bestmove
        );

        let mut builder = String::new();
        builder.push_str(&bestmove.0.to_string());
        builder.push_str(&bestmove.1.to_string());
        let args = vec![builder.as_str()];

        cmd::play(state, &args[..]);

        false
    }

    pub fn reset(state: &mut State, _args: &[&str]) -> bool {
        state.board = board::Board::new();
        state.notation.clear();

        print!("{}", state);
        false
    }

    pub fn load(state: &mut State, args: &[&str]) -> bool {
        if args.len() < 2 {
            println!("Error: please provide a game to load.");
            return false;
        }

        let board = board::Board::load(args[1]);
        match board {
            Ok(board) => {
                state.board = board;
                state.notation = String::from(args[1]);
                print!("{}", state);
            }
            Err(e) => {
                println!("{}", e);
            }
        }
        false
    }

    pub fn count(state: &mut State, _args: &[&str]) -> bool {
        println!("movecount: {}", state.board.movecount());
        false
    }

    pub fn take(state: &mut State, _args: &[&str]) -> bool {
        println!("takestreak: {}", state.board.takestreak());
        false
    }

    pub fn square(state: &mut State, _args: &[&str]) -> bool {
        match state.board.square() {
            Some(square) => println!("square: {}", square),
            None => println!("square: none"),
        }
        false
    }

    pub fn print(state: &mut State, _args: &[&str]) -> bool {
        print!("{}", state);
        false
    }

    pub fn quit(_state: &mut State, _args: &[&str]) -> bool {
        true
    }

    pub fn help(_state: &mut State, _args: &[&str]) -> bool {
        println!(concat!(
            "[0-4]<0-4>: make move\n",
            "u undo: undo last move\n",
            "e eval [timeout]: evaluate state\n",
            "b best [timeout]: make best move\n",
            "r reset: reset game\n",
            "l load: load game\n",
            "c count: print movecount\n",
            "t take: print takestreak\n",
            "s square: print square\n",
            "p print: print board\n",
            "q quit: quit the maker\n",
            "h help: show this help",
        ));
        false
    }
}

fn command(state: &mut State, prevcmd: &mut Option<Cmd>) -> bool {
    // Get the user command input
    print!("{} > ", state.board.onturn());
    io::stdout().flush().unwrap();

    let mut line = String::new();
    io::stdin().read_line(&mut line).unwrap();

    let cmd: Option<Cmd>;

    // Process the command
    let args: Vec<&str> = line.split_whitespace().collect();
    if args.len() > 0 {
        cmd = match args[0] {
            input if cmd::play_regex().is_match(input) => Some(cmd::play),
            "u" | "undo" => Some(cmd::undo),
            "e" | "eval" => Some(cmd::eval),
            "b" | "best" => Some(cmd::best),
            "r" | "reset" => Some(cmd::reset),
            "l" | "load" => Some(cmd::load),
            "c" | "count" => Some(cmd::count),
            "t" | "take" => Some(cmd::take),
            "s" | "square" => Some(cmd::square),
            "p" | "print" => Some(cmd::print),
            "q" | "quit" => Some(cmd::quit),
            "h" | "help" => Some(cmd::help),
            _ => {
                println!("Error: invalid command, see 'help'.");
                None
            }
        };

        // Update the prevcmd
        match args[0] {
            "u" | "undo" => *prevcmd = Some(cmd::undo),
            "b" | "best" => *prevcmd = Some(cmd::best),
            _ => *prevcmd = None,
        }
    } else {
        // Run the previous cmd if none was given
        cmd = *prevcmd;
        // TODO also run with prevargs
    }

    match cmd {
        Some(f) => f(state, &args[..]),
        None => false,
    }
}

fn main() {
    let mut prevcmd: Option<Cmd> = None;
    let mut state = State {
        board: board::Board::new(),
        notation: String::new(),
    };

    print!("{}", state.board);
    loop {
        if command(&mut state, &mut prevcmd) {
            break;
        }
    }
}
