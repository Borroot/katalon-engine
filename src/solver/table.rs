/// A fast lookup table without collision detection.
pub struct Table<T: Copy> {
    table: Vec<(u64, Option<T>)>,
    hits: usize, // used for statistics
}

impl<T: Copy> Table<T> {
    /// Create a new table of the given size.
    pub fn new(size: usize) -> Table<T> {
        Table {
            table: vec![(0, None); size],
            hits: 0,
        }
    }

    fn index(&self, key: u64) -> usize {
        (key % self.table.len() as u64) as usize
    }

    /// Put a new value with a given key inside the table.
    pub fn put(&mut self, key: u64, value: T) {
        let index = self.index(key);
        self.table[index] = (key, Some(value));
    }

    /// Retrieve the value identified by the given key, if present.
    pub fn get(&mut self, key: u64) -> Option<T> {
        if let (k, Some(value)) = self.table[self.index(key)] {
            if k == key {
                self.hits += 1;
                return Some(value);
            }
        }
        None
    }

    /// Retrieve how many hits this table has had.
    pub fn hits(&self) -> usize {
        self.hits
    }

    /// Retrieve the size of the table.
    pub fn size(&self) -> usize {
        self.table.len()
    }

    /// Retrieve the number of elements in the table.
    pub fn count(&self) -> usize {
        (0..self.table.len())
            .filter(|&i| self.table[i].1.is_some())
            .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn usage() {
        let mut table = Table::<usize>::new(10);

        assert_eq!(table.get(0), None);
        assert_eq!(table.get(3), None);
        assert_eq!(table.get(9), None);

        table.put(3, 3);
        assert_eq!(table.get(3), Some(3));

        table.put(15, 5);
        assert_eq!(table.get(15), Some(5));
        assert_eq!(table.get(5), None);

        table.put(1, 1);
        assert_eq!(table.get(1), Some(1));
        assert_eq!(table.get(211), None);

        table.put(13, 13);
        assert_eq!(table.get(13), Some(13));
        assert_eq!(table.get(3), None);
    }

    #[test]
    fn size_and_count() {
        let mut table = Table::<usize>::new(10);

        table.put(3, 3);
        table.put(15, 5);
        table.put(1, 1);
        table.put(13, 13);

        assert_eq!(table.count(), 3);
        assert_eq!(table.size(), 10);
    }

    #[test]
    fn hits() {
        let mut table = Table::<usize>::new(10);

        table.put(3, 3);
        table.put(15, 5);
        table.put(1, 1);
        table.put(13, 13);

        let _ = table.get(3);
        let _ = table.get(1);
        let _ = table.get(6);
        let _ = table.get(13);

        assert_eq!(table.hits(), 2);
    }
}
