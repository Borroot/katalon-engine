use katalon::{board, minmax, player};
use rand::prelude::*;
use regex;
use std::io::{self, Write};
use std::{fmt, time};

struct State {
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

trait Command {
    fn run(state: &mut State, args: &[&str]) -> bool;
}

struct Play;
struct Undo;
struct Eval;
struct Best;
struct Reset;
struct Load;
struct Count;
struct Take;
struct Print;
struct Quit;
struct Help;

impl Play {
    fn regex() -> regex::Regex {
        regex::Regex::new(r"^(?P<square>[0-4]?)(?P<cell>[0-4])$").unwrap()
    }
}

impl Command for Play {
    fn run(state: &mut State, args: &[&str]) -> bool {
        if state.board.isover() != None {
            println!("Warn: the game already finished.");
            return false;
        }

        let caps = Play::regex().captures(args[0]).unwrap();

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
}

impl Command for Undo {
    fn run(state: &mut State, _args: &[&str]) -> bool {
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
}

impl Command for Eval {
    fn run(state: &mut State, _args: &[&str]) -> bool {
        if state.board.isover() != None {
            println!("Warn: the game already finished.");
            return false;
        }

        // TODO add a timeout parameter

        let now = time::Instant::now();

        let (mut value, bestmoves) = minmax::Minmax::bestmoves(&state.board);
        value = minmax::Minmax::humanize_relative(state.board.movecount() as isize, value);

        println!(
            "evaluation: {} in {}ms\nmoves: {:?}",
            value,
            now.elapsed().as_millis(),
            bestmoves
        );

        false
    }
}

impl Command for Best {
    fn run(state: &mut State, _args: &[&str]) -> bool {
        if state.board.isover() != None {
            println!("Warn: the game already finished.");
            return false;
        }

        let now = time::Instant::now();

        let (mut value, bestmoves) = minmax::Minmax::bestmoves(&state.board);
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

        Play::run(state, &args[..]);

        false
    }
}

impl Command for Reset {
    fn run(state: &mut State, _args: &[&str]) -> bool {
        state.board = board::Board::new();
        state.notation.clear();

        print!("{}", state);
        false
    }
}

impl Command for Load {
    fn run(state: &mut State, args: &[&str]) -> bool {
        if args.len() == 0 {
            println!("Error: please provide a game to load.");
            return false;
        }

        let board = board::Board::load(args[0]);
        match board {
            Ok(board) => {
                state.board = board;
                state.notation = String::from(args[0]);
                print!("{}", state);
            }
            Err(e) => {
                println!("{}", e);
            }
        }
        false
    }
}

impl Command for Count {
    fn run(state: &mut State, _args: &[&str]) -> bool {
        println!("movecount: {}", state.board.movecount());
        false
    }
}

impl Command for Take {
    fn run(state: &mut State, _args: &[&str]) -> bool {
        println!("takestreak: {}", state.board.takestreak());
        false
    }
}

impl Command for Print {
    fn run(state: &mut State, _args: &[&str]) -> bool {
        print!("{}", state);
        false
    }
}

impl Command for Quit {
    fn run(_state: &mut State, _args: &[&str]) -> bool {
        true
    }
}

impl Command for Help {
    fn run(_state: &mut State, _args: &[&str]) -> bool {
        println!(concat!(
            "[0-4]<0-4>: make move\n",
            "u undo: undo last move\n",
            "e eval: evaluate state\n",
            "b best: make best move\n",
            "r reset: reset game\n",
            "l load: load game\n",
            "c count: print movecount\n",
            "t take: print takestreak\n",
            "p print: print board\n",
            "q quit: quit the maker\n",
            "h help: show this help",
        ));
        false
    }
}

fn command(state: &mut State, prevcmd: &mut Option<&str>) -> bool {
    // Get the user command input
    print!("{} > ", state.board.onturn());
    io::stdout().flush().unwrap();

    let mut line = String::new();
    io::stdin().read_line(&mut line).unwrap();

    let quit;

    // Process the command
    let args: Vec<&str> = line.split_whitespace().collect();
    if args.len() > 0 {
        quit = match args[0] {
            input if Play::regex().is_match(input) => Play::run(state, &args[0..]),
            "u" | "undo" => Undo::run(state, &args[1..]),
            "e" | "eval" => Eval::run(state, &args[1..]),
            "b" | "best" => Best::run(state, &args[1..]),
            "r" | "reset" => Reset::run(state, &args[1..]),
            "l" | "load" => Load::run(state, &args[1..]),
            "c" | "count" => Count::run(state, &args[1..]),
            "t" | "take" => Take::run(state, &args[1..]),
            "p" | "print" => Print::run(state, &args[1..]),
            "q" | "quit" => Quit::run(state, &args[1..]),
            "h" | "help" => Help::run(state, &args[1..]),
            _ => {
                println!("Error: invalid command, see 'help'.");
                false
            }
        };

        // Update the prevcmd
        match args[0] {
            "u" | "undo" => *prevcmd = Some("undo"),
            "b" | "best" => *prevcmd = Some("best"),
            _ => *prevcmd = None,
        }
    } else {
        // Run the previous cmd if none was given
        quit = match prevcmd {
            Some("undo") => Undo::run(state, &args[..]),
            Some("best") => Best::run(state, &args[..]),
            Some(_) | None => false,
        }
    }

    quit
}

fn main() {
    let mut state = State {
        board: board::Board::new(),
        notation: String::new(),
    };
    let mut prevcmd = None;

    print!("{}", state.board);
    loop {
        if command(&mut state, &mut prevcmd) {
            break;
        }
    }
}
