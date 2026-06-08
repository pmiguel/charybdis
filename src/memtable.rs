use crate::skiplist::SkipList;

const MT_TOMBSTONE_MARKER: [u8; 1] = [0];

pub struct MemTable {
    data: SkipList,
    pub is_frozen: bool
}

impl MemTable {
    pub fn new() -> Self {
        MemTable {
            data: SkipList::new(),
            is_frozen: false
        }
    }

    pub fn get(&self, search_key: &[u8]) -> Option<&[u8]> {
        match self.data.get(search_key) {
            Some(data) => {
                if data == MT_TOMBSTONE_MARKER {
                    return None;
                }
                Some(data)
            }
            None => None
        }
    }

    pub fn put(&mut self, search_key: &[u8], data: &[u8]) -> Result<(), &str>{
        if self.is_frozen {
            return Err("Can't update frozen MemTable");
        }
        self.data.put(search_key, data);
        Ok(())
    }

    pub fn del(&mut self, search_key: &[u8]) -> Result<(), &str>{
        if self.is_frozen {
            return Err("Can't update frozen MemTable");
        }
        self.data.put(search_key, &MT_TOMBSTONE_MARKER.as_slice());
        Ok(())
    }

    pub fn freeze(&mut self) {
        self.is_frozen = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_fetches_existing_key() {
        let mut mem_table = MemTable::new();
        mem_table.put(b"apple", b"red").unwrap();
        assert_eq!(mem_table.get(b"apple"), Some(&b"red"[..]));
    }

    #[test]
    fn test_get_none_on_non_existing() {
        let mem_table = MemTable::new();
        assert_eq!(mem_table.get(b"apple"),None);
    }

    #[test]
    fn test_put_creates_key() {
        let mut mem_table = MemTable::new();

        assert_eq!(mem_table.get(b"apple"), None);
        mem_table.put(b"apple", b"red").unwrap();
        assert_eq!(mem_table.get(b"apple"), Some(&b"red"[..]));
    }

    #[test]
    fn test_put_updates_key() {
        let mut mem_table = MemTable::new();
        mem_table.put(b"apple", b"red").unwrap();
        assert_eq!(mem_table.get(b"apple"), Some(&b"red"[..]));

        mem_table.put(b"apple", b"green").unwrap();
        assert_eq!(mem_table.get(b"apple"), Some(&b"green"[..]));
    }

    #[test]
    fn test_del_removes_key() {
        let mut mem_table = MemTable::new();
        mem_table.put(b"apple", b"red").unwrap();
        assert_eq!(mem_table.get(b"apple"), Some(&b"red"[..]));
        mem_table.del(b"apple").unwrap();
        assert_eq!(mem_table.get(b"apple"), None);
    }

    #[test]
    fn test_freeze_marks_memtable_frozen() {
        let mut mem_table = MemTable::new();
        mem_table.freeze();

        assert_eq!(mem_table.is_frozen, true);
    }

    #[test]
    fn test_freeze_cant_put() {
        let mut mem_table = MemTable::new();
        mem_table.put(b"apple", b"green").unwrap();

        mem_table.freeze();

        let result = mem_table.put(b"apple", b"red");

        assert_eq!(result, Err("Can't update frozen MemTable"));
    }

    #[test]
    fn test_freeze_cant_delete() {
        let mut mem_table = MemTable::new();
        mem_table.put(b"apple", b"green").unwrap();

        mem_table.freeze();

        let result = mem_table.del(b"apple");
        assert_eq!(result, Err("Can't update frozen MemTable"));
    }

    #[test]
    fn test_freeze_can_read() {
        let mut mem_table = MemTable::new();
        mem_table.put(b"apple", b"green").unwrap();

        mem_table.freeze();

        let result = mem_table.get(b"apple").unwrap();
        assert_eq!(result, b"green");
    }
}