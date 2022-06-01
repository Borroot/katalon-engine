use katalon::{board, eval, solver};
use std::fs::File;
use std::io::{BufRead, BufReader};
use tabled::{Tabled, Table};

/// The difficulty for the benchmark data.
/// Depth 20
/// - End medium:    30-  moves
/// - End easy:      5-30 moves
/// Depth 10
/// - Middle medium: 30-  moves
/// - Middle easy:   5-30 moves
/// Depth 5
/// - Begin hard:    30-  moves
/// - Begin medium:  5-30 moves

/// Test set data meta information
/// - timeout at 10 seconds
/// - 20 entries each
/// - (so max 10x20x6 secs = 20 mins total)

/// Test set data format
/// e.g. depth20_low.txt
///   44023411421 loss 10
///   21442341040 win 9

/// Table headers
/// | solver | test set | mean time | max time | mean visited | max visited | visited / s |

struct Entry {
    pub board: board::Board,
    pub eval: eval::Eval,
}

#[derive(Debug, Tabled)]
struct Bench {
    pub test_set: String,
    pub timeouts: usize,
    pub mean_time: u128,
    pub max_time: u128,
    pub mean_visited: usize,
    pub max_visited: usize,
    pub visited_per_second: usize,
}

fn load_entry(entry: String) -> Entry {
    let mut entry: std::str::Split<&str> = entry.split(" ");

    let board = board::Board::load(&entry.next().unwrap()).unwrap();
    let result = match entry.next().unwrap() {
        "win" => eval::Result::Win,
        "loss" => eval::Result::Loss,
        "draw" => eval::Result::Draw,
        &_ => panic!(),
    };
    let distance = entry.next().unwrap().parse::<u8>().unwrap();
    let eval = eval::Eval::new(result, distance);

    Entry { board, eval }
}

fn load_file(filename: String) -> Vec<Entry> {
    let file = File::open(&filename).expect(&format!("Could not open file {}.", &filename));
    let reader = BufReader::new(file);

    let mut entries = Vec::<Entry>::new();
    for line in reader.lines() {
        entries.push(load_entry(line.unwrap()));
    }
    entries
}

fn run_set(name: String, entries: Vec<Entry>) -> Result<Bench, ()> {
    let mut time = Vec::<std::time::Duration>::new();
    let mut visited = Vec::<usize>::new();

    for (index, entry) in entries.iter().enumerate() {
        let (result, stats) =
            solver::bestmoves_with_stats(&entry.board, std::time::Duration::from_secs(10));

        if let Ok((eval, _)) = result {
            if eval != entry.eval {
                println!(
                    "error in {}: wrong eval of {}, should be {}",
                    index, eval, entry.eval
                );
            }
            time.push(stats.time);
            visited.push(stats.visited);
        }
    }

    time.sort();
    visited.sort();

    //println!("time: {:?}", time);
    //println!("visited: {:?}", visited);

    let sum_time: f64 = time.iter().map(|t| t.as_millis()).sum::<u128>() as f64;
    let sum_visited: f64 = visited.iter().sum::<usize>() as f64;
    let visited_per_second = ((sum_visited / sum_time) * 1000.0) as usize;

    if time.len() > 0 {
        Ok(Bench {
            test_set: name,
            timeouts: entries.len() - time.len(),
            mean_time: time[time.len() / 2].as_millis(),
            max_time: time[time.len() - 1].as_millis(),
            mean_visited: visited[visited.len() / 2],
            max_visited: visited[visited.len() - 1],
            visited_per_second,
        })
    } else {
        Err(())
    }
}

fn main() {
    let variants = ["low", "high"];
    let depths = [20, 10];

    let mut benches = Vec::<Bench>::new();
    for depth in depths {
        for variant in variants {
            let filename = format!("res/benchmark/depth{}_{}.txt", depth, variant);
            let entries = load_file(filename);

            println!("starting benchmark of depth {} {}", depth, variant);
            if let Ok(bench) = run_set(format!("depth {} {}", depth, variant), entries) {
                benches.push(bench);
            }
        }
    }
    println!("{}", Table::new(benches).to_string());
}
