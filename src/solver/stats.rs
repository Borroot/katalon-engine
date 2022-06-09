use super::table;
use crate::eval;

/// The statistics of a given search performed by the solver.
#[derive(Debug)]
pub struct Stats {
    /// Used to determine the time the solver takes.
    now: std::time::Instant,
    /// The amount of time the solver took.
    pub time: std::time::Duration,

    /// The number of states that were evaluated.
    pub visited: usize,
    /// Table information.
    pub table: TableStats,
}

/// Statistics on a table.
#[derive(Debug)]
pub struct TableStats {
    /// Size of the table.
    pub size: usize,
    /// Number of elements in the table.
    pub count: usize,
    /// #elements / size
    pub sparcity: f64,
    /// Number of hits made.
    pub hits: usize,
}

impl Stats {
    pub fn new() -> Self {
        Self {
            now: std::time::Instant::now(),
            time: std::time::Duration::ZERO,
            visited: 0,
            table: TableStats {
                size: 0,
                count: 0,
                sparcity: 0.0,
                hits: 0,
            },
        }
    }

    pub fn end(&mut self) {
        self.time = self.now.elapsed();
    }

    /// Increase the number of nodes that are visited in the search.
    pub fn add_visited(&mut self) {
        self.visited += 1;
    }

    // Add information of the transposition table.
    //pub fn add_table(&mut self, table: &table::Table) {
    //self.table = TableStats {
    //size: table.size(),
    //count: table.count(),
    //sparcity: table.count() as f64 / table.size() as f64,
    //hits: table.hits(),
    //};
    //}
}

impl std::fmt::Display for Stats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            concat!("states visited: {}\n", "time: {}ms\n", "{}\n",),
            self.visited,
            self.time.as_millis(),
            self.table,
        )
    }
}

impl std::fmt::Display for TableStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            concat!("count / size = {} / {} = {:.6}\n", "table hits: {}\n",),
            self.count, self.size, self.sparcity, self.hits,
        )
    }
}
