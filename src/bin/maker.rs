use katalon::board::*;
use regex::Regex;
use std::io::{self, Write};

struct State {
    board: Board,
    notation: String,
}

impl State {
    fn new() -> Self {
        return State {
            board: Board::new(),
            notation: String::new(),
        };
    }

    fn play_explicit(&mut self, square: u8, cell: u8) {
        self.board.play_explicit(square, cell);
        self.notation.push_str(&square.to_string());
        self.notation.push_str(&cell.to_string());
    }

    fn play(&mut self, cell: u8) {
        self.board.play(cell);
        self.notation.push_str(&cell.to_string());
    }

    fn undo(&mut self) {
        match self.board.movecount() {
            0 => return,
            1 => {
                self.notation.clear();
                self.board = Board::new();
            },
            _ => {
                self.notation.pop();
                self.board = Board::load(&self.notation).unwrap();
            }
        }
    }

    fn reset(&mut self) {
        self.board = Board::new();
        self.notation.clear();
    }

    fn load(&mut self, moves: &str) -> bool {
        let board = Board::load(moves);
        match board {
            Ok(board) => {
                self.board = board;
                self.notation = String::from(moves);
                return true;
            }
            Err(e) => {
                println!("{}", e);
                return false;
            }
        }
    }
}

fn play(caps: regex::Captures<'_>, state: &mut State) {
    let cell = caps.name("cell").unwrap().as_str().chars().next().unwrap();
    let cell = cell as u8 - '0' as u8;

    if let Some(square) = caps.name("square").unwrap().as_str().chars().next() {
        if state.board.movecount() != 0 {
            println!("Error: please only provide the cell.");
            return;
        }

        let square = square as u8 - '0' as u8;
        if !state.board.canplay_explicit(square, cell) {
            println!("Error: illegal move.");
            return;
        }

        state.play_explicit(square, cell);
    } else {
        if state.board.movecount() == 0 {
            println!("Error: please also provide the square.");
            return;
        }

        if !state.board.canplay(cell) {
            println!("Error: illegal move.");
            return;
        }

        state.play(cell);
    }
    println!("{}= {}", state.board, state.notation);
}

fn undo(state: &mut State) {
    state.undo();
    show(state);
}

fn reset(state: &mut State) {
    state.reset();
    print!("{}", state.board);
}

fn load(args: &Vec<&str>, state: &mut State) {
    if args.len() < 2 {
        println!("Error: please provide a game to load.");
        return;
    }

    if state.load(args[1]) {
        print!("{}", state.board);
    }
}

fn show(state: &mut State) {
    if state.board.movecount() == 0 {
        print!("{}", state.board);
    } else {
        println!("{}= {}", state.board, state.notation);
    }
}

fn help() {
    println!(concat!(
        "[0-4]<0-4>: make move\n",
        "u undo: undo last move\n",
        "n new: new game\n",
        "l load: load game\n",
        "s show: show board\n",
        "q quit: quit the maker\n",
        "h help: show this help",
    ));
}

fn parse(line: String, state: &mut State) -> bool {
    let args: Vec<&str> = line.split_whitespace().collect();
    let re = Regex::new(r"^(?P<square>[0-4]?)(?P<cell>[0-4])$").unwrap();

    if args.len() > 0 {
        match args[0] {
            nums if re.is_match(nums) => play(re.captures(nums).unwrap(), state),
            "u" | "undo" => undo(state),
            "n" | "new" => reset(state),
            "l" | "load" => load(&args, state),
            "s" | "show" => show(state),
            "q" | "quit" => return true,
            "h" | "help" => help(),
            _ => println!("Error: invalid command, see 'help'."),
        }
    }
    return false;
}

fn input() -> String {
    let mut line = String::new();

    print!("> ");
    io::stdout().flush().unwrap();

    io::stdin().read_line(&mut line).unwrap();
    return line;
}

fn main() {
    let mut state = State::new();
    print!("{}", state.board);

    loop {
        let line = input();
        if parse(line, &mut state) {
            break;
        }
    }
}
