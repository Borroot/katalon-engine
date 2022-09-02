use super::table;

/// The statistics of a given search performed by the solver.
pub struct Stats {
    /// The time the solver started, used to calculate the total time.
    time_start: Option<std::time::Instant>,

    /// The amount of time the solver took in total.
    pub time: std::time::Duration,
    /// If the timeout was reached.
    pub timeout: bool,

    /// The number of null window searches iterations.
    pub nullwindows: usize,
    /// The number of states that were evaluated.
    pub visited: usize,

    /// Table information.
    pub table: table::Stats,
}

impl Stats {
    pub fn new() -> Self {
        Self {
            time_start: None,
            time: std::time::Duration::ZERO,
            timeout: false,
            nullwindows: 0,
            visited: 0,
            table: table::Stats::new(),
        }
    }

    /// A timeout has been reached.
    pub fn timeout(&mut self) {
        self.timeout = true;
    }

    /// Add the table stats.
    pub fn add_table(&mut self, stats: table::Stats) {
        self.table = stats;
    }

    /// Start the stopwatch, this should be called when the search starts.
    pub fn stopwatch_start(&mut self) {
        self.time_start = Some(std::time::Instant::now());
    }

    /// Stop the stopwatch, this should be called when the search stopped.
    pub fn stopwatch_stop(&mut self) {
        self.time = self
            .time_start
            .expect("You forgot to call stopwatch_start().")
            .elapsed();
    }
}

impl std::fmt::Display for Stats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "search time: {}ms", self.time.as_millis())?;
        match self.timeout {
            true => write!(f, " TIMEOUT\n")?,
            false => write!(f, "\n")?,
        }

        if self.nullwindows > 0 {
            write!(f, "null windows: {}\n", self.nullwindows)?;
        }

        write!(
            f,
            concat!(
                "states visited: {}\n",
                "\n{}\n",
            ),
            self.visited,
            self.table,
        )
    }
}
