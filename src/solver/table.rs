use crate::eval;

/// A fast lookup table without collision detection.
pub struct Table {
    table: Vec<Option<Entry>>,
}

/// A flag which indicates whether the entry is an upperbound, lowerbound or exact value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Flag {
    UPPERBOUND,
    LOWERBOUND,
    EXACT,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Entry {
    key: u64,
    value: eval::Eval,
    flag: Flag,
    bestmove: u8,
    // TODO add movecount for performing the replacement strategy
    // https://www.chessprogramming.org/Transposition_Table
}

impl Table {
    /// Create a new table of at least the given size.
    /// The size of a table is always the closest bigger than the given size prime.
    pub fn new(size: usize) -> Self {
        let size = primal::Primes::all().find(|p| p >= &size).unwrap();
        Self {
            table: vec![None; size],
        }
    }

    /// Create a table of the given amount of gigabytes.
    /// Be careful not to make it too big.
    pub fn from_gb(size: f32) -> Self {
        let size = (size * 1.0e9) as usize / std::mem::size_of::<Entry>();
        Self::new(size)
    }

    fn index(&self, key: u64) -> usize {
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
                return Some(entry);
            }
        }
        None
    }

    /// Retrieve the size of the table.
    pub fn size(&self) -> usize {
        self.table.len()
    }

    /// Retrieve the number of elements in the table.
    pub fn count(&self) -> usize {
        (0..self.table.len())
            .filter(|&i| self.table[i].is_some())
            .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gigabyte() {
        let gb = 0.7;
        let table = Table::from_gb(gb);
        let size = table.size();
        let bytes = size * std::mem::size_of::<Entry>();

        assert!(bytes < (1e9 * gb + 1000.0) as usize);
        assert!(bytes > (1e9 * gb - 1000.0) as usize);
    }

    #[test]
    fn put_and_get() {
        let mut table = Table::new(10);
        assert_eq!(table.size(), 11);

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
