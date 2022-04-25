use cmd::Cmd;
use katalon::{board, input, player::Player, random, solver};
use rand::Rng;

pub struct State {
    board: board::Board,
    notation: String,
}

impl std::fmt::Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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

    pub fn play(state: &mut State, args: &[&str]) -> bool {
        if state.board.isover() != None {
            println!("Warn: the game already finished.");
            return false;
        }

        let extracted = input::extract(&state.board, &args[0]);
        if let Err(e) = extracted {
            println!("{}", e);
            return false;
        }
        let (square, cell) = extracted.unwrap();

        // Update the board and notation
        if state.board.isfirst() {
            state.notation.push_str(&square.to_string());
        }
        state.notation.push_str(&cell.to_string());
        state.board.play(square, cell);

        print!("{}", state);

        match state.board.isover() {
            Some(board::Result::Draw) => println!("It's a draw!"),
            Some(result) => println!("Player {} won!", result.player().unwrap()),
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

        // TODO cleanup eval and best function, e.g. so no duplicate code
        let now = std::time::Instant::now();

        let result = {
            if timeout == None {
                Ok(solver::bestmoves(&state.board))
            } else {
                solver::bestmoves_timeout(
                    &state.board,
                    std::time::Duration::from_secs(timeout.unwrap()),
                )
            }
        };

        if result.is_err() {
            println!("timeout after {}s", timeout.unwrap());
            return false;
        }

        let (value, bestmoves) = result.unwrap();

        println!(
            "evaluation: {} ({}ms)\nmoves: {:?}",
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

        let now = std::time::Instant::now();

        let result = {
            if timeout == None {
                Ok(solver::bestmoves(&state.board))
            } else {
                solver::bestmoves_timeout(
                    &state.board,
                    std::time::Duration::from_secs(timeout.unwrap()),
                )
            }
        };

        if result.is_err() {
            println!("timeout after {}s", timeout.unwrap());
            return false;
        }

        let (value, bestmoves) = result.unwrap();
        let mut rng = rand::thread_rng();
        let bestmove = bestmoves[rng.gen_range(0..bestmoves.len()) as usize];

        println!(
            "evaluation: {} ({}ms)\n{:?} -> {:?}",
            value,
            now.elapsed().as_millis(),
            bestmoves,
            bestmove
        );

        let builder = format!("{}{}", bestmove.0, bestmove.1);
        let args = vec![builder.as_str()];

        cmd::play(state, &args[..]);

        false
    }

    pub fn random(state: &mut State, _args: &[&str]) -> bool {
        if state.board.isover() != None {
            println!("Warn: the game already finished.");
            return false;
        }

        let (square, cell) = random::Random.play(&state.board);
        let builder = format!("{}{}", square, cell);

        cmd::play(state, &[&builder.as_str()]);

        false
    }

    pub fn new(state: &mut State, _args: &[&str]) -> bool {
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
            "r random: make random move\n",
            "n new: new game\n",
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
    let line = input::request(format!("{} > ", state.board.onturn()));
    let cmd: Option<Cmd>;

    // Process the command
    let args: Vec<&str> = line.split_whitespace().collect();
    if args.len() > 0 {
        cmd = match args[0] {
            input if input::move_regex().is_match(input) => Some(cmd::play),
            "u" | "undo" => Some(cmd::undo),
            "e" | "eval" => Some(cmd::eval),
            "b" | "best" => Some(cmd::best),
            "r" | "random" => Some(cmd::random),
            "n" | "new" => Some(cmd::new),
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
            "r" | "random" => *prevcmd = Some(cmd::random),
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
