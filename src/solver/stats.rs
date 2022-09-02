use super::table;

/// The statistics of a given search performed by the solver.
pub struct Stats {
    /// The amount of time the solver took in total.
    pub time: std::time::Duration,
    /// The number of states that were evaluated.
    pub visited: usize,
    /// The reached search depth in iterative deepening.
    pub depth: usize,
    /// If the timeout was reached.
    pub timeout: bool,
    /// Table information.
    pub table: table::Stats,
}

impl Stats {
    pub fn new() -> Self {
        Self {
            time: std::time::Duration::ZERO,
            visited: 0,
            depth: 0,
            timeout: false,
            table: table::Stats::new(),
        }
    }
}

impl std::fmt::Display for Stats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            concat!(
                "total search time: {}ms\n",
                "states visited: {}\n",
                "depth reached: {}\n",
                "timeout: {}\n",
                "correct: {}\n",
                "{}\n",
            ),
            self.time.as_millis(),
            self.visited,
            self.depth,
            self.timeout,
            self.table,
        )
    }
}
