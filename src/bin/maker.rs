use katalon::{board, minmax, player};
use regex;
use std::fmt;
use std::io::{self, Write};

struct State {
    board: board::Board,
    notation: String,
    finished: bool,
}

impl State {
    fn new() -> Self {
        return State {
            board: board::Board::new(),
            notation: String::new(),
            finished: false,
        };
    }

    fn play(&mut self, square: u8, cell: u8) {
        if self.board.isfirst() {
            self.notation.push_str(&square.to_string());
        }
        self.notation.push_str(&cell.to_string());
        self.board.play(square, cell);
    }

    fn undo(&mut self) {
        match self.board.movecount() {
            0 => return,
            1 => {
                self.notation.clear();
                self.board = board::Board::new();
            }
            _ => {
                self.finished = false;
                self.notation.pop();
                self.board = board::Board::load(&self.notation).unwrap();
            }
        }
    }

    fn reset(&mut self) {
        self.finished = false;
        self.board = board::Board::new();
        self.notation.clear();
    }

    fn load(&mut self, moves: &str) -> bool {
        let board = board::Board::load(moves);
        match board {
            Ok(board) => {
                self.finished = self.board.isover() != None;
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

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.board.isfirst() {
            write!(f, "{}", self.board)
        } else {
            write!(f, "{}= {}\n", self.board, self.notation)
        }
    }
}

fn play(caps: regex::Captures<'_>, state: &mut State) {
    if state.finished {
        println!("Warn: the game already finished.");
        return;
    }

    let cell = caps.name("cell").unwrap().as_str().chars().next().unwrap();
    let cell = cell as u8 - '0' as u8;

    // TODO make that you can also specify the square when not necessary
    // copy from human player

    if let Some(square) = caps.name("square").unwrap().as_str().chars().next() {
        if !state.board.isfirst() {
            println!("Error: please only provide the cell.");
            return;
        }

        let square = square as u8 - '0' as u8;
        if !state.board.canplay(square, cell) {
            println!("Error: illegal move.");
            return;
        }

        state.play(square, cell);
    } else {
        if state.board.isfirst() {
            println!("Error: please also provide the square.");
            return;
        }

        if !state.board.canplay(state.board.square().unwrap(), cell) {
            println!("Error: illegal move.");
            return;
        }

        state.play(state.board.square().unwrap(), cell);
    }
    print!("{}", state);

    if state.board.isover() != None {
        state.finished = true;
    }
    match state.board.isover() {
        Some(board::Result::Player1) => println!("Player {} won!", player::Players::Player1),
        Some(board::Result::Player2) => println!("Player {} won!", player::Players::Player2),
        Some(board::Result::Draw) => println!("It's a draw!"),
        None => (),
    }
}

fn undo(state: &mut State) {
    state.undo();
    print!("{}", state);
}

fn evaluate(board: &board::Board) {
    // TODO add a timeout parameter
    // TODO print time it takes
    let (mut value, bestmoves) = minmax::Minmax::bestmoves(board);
    value = minmax::Minmax::humanize_relative(board.movecount() as isize, value);
    println!("eval: {}, moves: {:?}", value, bestmoves);
}

fn reset(state: &mut State) {
    state.reset();
    print!("{}", state);
}

fn load(args: &Vec<&str>, state: &mut State) {
    if args.len() < 2 {
        println!("Error: please provide a game to load.");
        return;
    }

    if state.load(args[1]) {
        print!("{}", state);
    }
}

fn help() {
    println!(concat!(
        "[0-4]<0-4>: make move\n",
        "u undo: undo last move\n",
        "e eval: evaluate state\n",
        "n new: new game\n",
        "l load: load game\n",
        "c count: print movecount\n",
        "t takestreak: print takestreak\n",
        "p print: print board\n",
        "q quit: quit the maker\n",
        "h help: show this help",
    ));
}

fn parse(line: String, state: &mut State) -> bool {
    let args: Vec<&str> = line.split_whitespace().collect();
    let re = regex::Regex::new(r"^(?P<square>[0-4]?)(?P<cell>[0-4])$").unwrap();

    // TODO add history, repeat undo with enter
    // TODO undo [num] add optional number of undo's
    // TODO add 'best AI move'
    if args.len() > 0 {
        match args[0] {
            nums if re.is_match(nums) => play(re.captures(nums).unwrap(), state),
            "u" | "undo" => undo(state),
            "e" | "eval" => evaluate(&state.board),
            "n" | "new" => reset(state),
            "l" | "load" => load(&args, state),
            "c" | "count" => println!("movecount: {}", state.board.movecount()),
            "t" | "takestreak" => println!("takestreak: {}", state.board.takestreak()),
            "p" | "print" => print!("{}", state),
            "q" | "quit" => return true,
            "h" | "help" => help(),
            _ => println!("Error: invalid command, see 'help'."),
        }
    }
    return false;
}

fn input(player: &str) -> String {
    print!("{} > ", player);
    io::stdout().flush().unwrap();

    let mut line = String::new();
    io::stdin().read_line(&mut line).unwrap();

    return line;
}

fn main() {
    let mut state = State::new();
    print!("{}", state.board);

    loop {
        let line = input(&state.board.onturn().to_string());
        if parse(line, &mut state) {
            break;
        }
    }
}
