use crate::{eval, stats::table};

/// A fast lookup table without collision detection.
pub struct Table {
    table: Vec<Option<Entry>>,
    stats: table::Stats,
}

/// A flag which indicates whether the entry is an upperbound, lowerbound or exact value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Flag {
    /// The value is an upperbound and can be used to set beta.
    UPPERBOUND,
    /// The value is an lowerbound and can be used to set alpha.
    LOWERBOUND,
    /// The value is exact and can immediately be returned.
    EXACT,
}

/// One entry in the table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Entry {
    /// A key uniquely identifying the board state, used to index the table.
    pub key: u64,
    /// The current evaluation of the board.
    pub value: eval::Eval,
    /// Flag indicating what type of value this is.
    pub flag: Flag,
    /// The bestmove that can be made from this position.
    pub bestmove: u8,
}

impl Table {
    /// Create a new table of at least the given size.
    /// The size of a table is always the closest bigger than the given size prime.
    pub fn new(size: usize) -> Self {
        let now = std::time::Instant::now();

        let size = primal::Primes::all().find(|p| p >= &size).unwrap();
        let mut table = Self {
            table: vec![None; size],
            stats: table::Stats::new(),
        };

        table.stats.time = now.elapsed();
        table.stats.size = size;
        table
    }

    /// Create a table of the given amount of gigabytes.
    /// Be careful not to make it too big.
    pub fn from_gb(size: f32) -> Self {
        let size = (size * 1.0e9) as usize / std::mem::size_of::<Entry>();
        Self::new(size)
    }

    fn index(&self, key: u64) -> usize {
        // TODO test key hashing for a more uniform distribution, e.g. seahash
        (key % self.table.len() as u64) as usize
    }

    /// Put a new value with a given key inside the table.
    pub fn put(&mut self, key: u64, value: eval::Eval, flag: Flag, bestmove: u8) {
        let entry = Entry {
            key,
            value,
            flag,
            bestmove,
        };
        let index = self.index(entry.key);
        self.table[index] = Some(entry);
    }

    /// Retrieve the value identified by the given key, if present.
    pub fn get(&mut self, key: u64) -> Option<Entry> {
        if let Some(entry) = self.table[self.index(key)] {
            if entry.key == key {
                self.stats.hits += 1;
                return Some(entry);
            }
        }
        None
    }

    /// Retrieve the number of elements in the table.
    fn count(&self) -> usize {
        (0..self.table.len())
            .filter(|&i| self.table[i].is_some())
            .count()
    }

    /// Retrieve the stats of this table.
    pub fn stats(&mut self) -> table::Stats {
        self.stats.count = self.count();
        self.stats.sparcity = self.stats.count as f64 / self.stats.size as f64;
        self.stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test the creation of a table by specifying the number of gigabytes.
    #[test]
    fn gigabyte() {
        let gb = 0.7;
        let table = Table::from_gb(gb);
        let size = table.table.len();
        let bytes = size * std::mem::size_of::<Entry>();

        assert!(bytes < (1e9 * gb + 1000.0) as usize);
        assert!(bytes > (1e9 * gb - 1000.0) as usize);
    }

    /// Test whether the put and get functions work correctly.
    #[test]
    fn put_and_get() {
        let mut table = Table::new(10);
        assert_eq!(table.table.len(), 11);

        assert_eq!(table.get(0), None);
        assert_eq!(table.get(3), None);
        assert_eq!(table.get(9), None);

        let entry = Entry {
            key: 3,
            value: eval::Eval::MIN,
            flag: Flag::UPPERBOUND,
            bestmove: 0,
        };
        table.put(entry.key, entry.value, entry.flag, entry.bestmove);
        assert_eq!(table.get(3), Some(entry));
        assert_eq!(table.get(14), None);

        let entry = Entry {
            key: 8,
            value: eval::Eval::MAX,
            flag: Flag::LOWERBOUND,
            bestmove: 3,
        };
        table.put(entry.key, entry.value, entry.flag, entry.bestmove);
        assert_eq!(table.get(8), Some(entry));
        assert_eq!(table.get(30), None);

        let entry = Entry {
            key: 19,
            value: eval::Eval::MAX,
            flag: Flag::LOWERBOUND,
            bestmove: 3,
        };
        table.put(entry.key, entry.value, entry.flag, entry.bestmove);
        assert_eq!(table.get(19), Some(entry));
        assert_eq!(table.get(8), None);
    }

    /// Test whether the size and count functions work correctly.
    #[test]
    fn size_and_count() {
        let mut table = Table::new(10);

        let entry = Entry {
            key: 3,
            value: eval::Eval::MIN,
            flag: Flag::UPPERBOUND,
            bestmove: 0,
        };
        table.put(entry.key, entry.value, entry.flag, entry.bestmove);

        let entry = Entry {
            key: 8,
            value: eval::Eval::MAX,
            flag: Flag::LOWERBOUND,
            bestmove: 3,
        };
        table.put(entry.key, entry.value, entry.flag, entry.bestmove);

        let entry = Entry {
            key: 19,
            value: eval::Eval::MAX,
            flag: Flag::LOWERBOUND,
            bestmove: 3,
        };
        table.put(entry.key, entry.value, entry.flag, entry.bestmove);

        assert_eq!(table.count(), 2);
    }
}
