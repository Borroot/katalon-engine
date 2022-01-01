// TODO Add collision detection so all states are saved.

pub struct Table<T: Copy> {
    // TODO remove pub here
    pub table: Vec<(u64, Option<T>)>,
}

impl<T: Copy> Table<T> {
    pub fn new(size: usize) -> Table<T> {
        Table {
            table: vec![(0, None); size],
        }
    }

    fn index(&self, key: u64) -> usize {
        (key % self.table.len() as u64) as usize
    }

    pub fn put(&mut self, key: u64, value: T) {
        let index = self.index(key);
        self.table[index] = (key, Some(value));
    }

    pub fn get(&self, key: u64) -> Option<T> {
        if let (k, Some(value)) = self.table[self.index(key)] {
            if k == key {
                return Some(value);
            }
        }
        None
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
}
