/// Statistics on a table.
#[derive(Clone, Copy)]
pub struct Stats {
    /// Time it took to create the table.
    pub time: std::time::Duration,
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
            time: std::time::Duration::ZERO,
            size: 0,
            count: 0,
            sparcity: 0.0,
            hits: 0,
        }
    }
}

impl std::fmt::Display for Stats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            concat!(
                "table creation time: {}ms\n",
                "table hits: {}\n",
                "count / size = {} / {} = {:.6}\n",
            ),
            self.time.as_millis(),
            self.hits,
            self.count,
            self.size,
            self.sparcity,
        )
    }
}
