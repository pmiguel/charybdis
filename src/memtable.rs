use crate::skiplist::SkipList;

const MT_TOMBSTONE_MARKER: [u8; 1] = [0];

pub struct MemTable {
    data: SkipList,
}

impl MemTable {
    pub fn new() -> Self {
        MemTable {
            data: SkipList::new()
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
            None => { None }
        }
    }

    pub fn put(&mut self, search_key: &[u8], data: &[u8]) {
        self.data.put(search_key, data);
    }

    pub fn del(&mut self, search_key: &[u8]) {
        // tombstone
        self.data.put(search_key, MT_TOMBSTONE_MARKER.as_slice());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_put_get_and_del() {
        let mut list = MemTable::new();
        list.put(b"apple", b"red");
        list.put(b"banana", b"yellow");

        assert_eq!(list.get(b"apple"), Some(&b"red"[..]));
        assert_eq!(list.get(b"banana"), Some(&b"yellow"[..]));
        assert_eq!(list.get(b"grape"), None);

        list.del(b"apple");
        assert_eq!(list.get(b"apple"), None);
    }
}