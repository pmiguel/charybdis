const SL_MAX_LEVEL: usize = 16;
const SL_FLIP_PROB: f64 = 0.5;

pub struct Node {
    key: Vec<u8>,
    value: Vec<u8>,
    next: Vec<Option<usize>>
}

pub struct SkipList {
    arena: Vec<Node>,
    head_idx: usize,
    top_level: usize,
    pub size_bytes: usize,
    length: usize
}

impl Node {
    pub fn size(&self) -> usize {
        let key_size = self.key.len();
        let val_size = self.value.len();
        let next_size = self.next.len() * size_of::<Option<usize>>();

        key_size + val_size + next_size
    }
}

impl SkipList {
    pub fn new() -> Self {
        let head = Node{
            key: vec![],
            value: vec![],
            next: vec![None; SL_MAX_LEVEL]
        };

        let head_size = head.size();

        SkipList {
            arena: vec![head],
            head_idx: 0,
            top_level: 0,
            length: 0,
            size_bytes: head_size
        }
    }

    pub fn get(&self, search_key: &[u8]) -> Option<&[u8]> {
        let mut current_idx = self.head_idx;
        for current_level in (0..=self.top_level).rev() {
            loop {
                let next = self.arena[current_idx].next[current_level];
                match next {
                    None => { break; }
                    Some(next_idx) => {
                        // equivalent to &self.arena[next_idx].key[..]
                        let next_key = self.arena[next_idx].key.as_slice();
                        if next_key == search_key {
                            return Some(self.arena[next_idx].value.as_slice())
                        } else if next_key < search_key {
                            current_idx = next_idx;
                        } else {
                            break;
                        }
                    }
                }
            }
        }
        None
    }

    fn flip() -> usize {
        let mut level: usize = 0;
        while rand::random_bool(SL_FLIP_PROB) && level < SL_MAX_LEVEL - 1 {
            level += 1;
        }
        level
    }

    pub fn put(&mut self, search_key: &[u8], value: &[u8]) -> () {
        let mut update = vec![self.head_idx; SL_MAX_LEVEL];
        let mut current_idx = self.head_idx;

        for current_level in (0..=self.top_level).rev() {
            loop {
                let next = self.arena[current_idx].next[current_level];
                match next {
                    None => {
                        update[current_level] = current_idx;
                        break;
                    }
                    Some(next_idx) => {
                        let next_key = self.arena[next_idx].key.as_slice();
                        if next_key == search_key {

                            // Size accounting on update
                            let old_size = self.arena[next_idx].value.len();
                            let new_size = value.len();
                            self.update_value_size_bytes(old_size, new_size);

                            self.arena[next_idx].value = value.into();
                            return
                        } else if next_key < search_key {
                            current_idx = next_idx;
                        } else {
                            update[current_level] = current_idx;
                            break;
                        }
                    }
                }
            }
        }
        let target_level = Self::flip();
        if target_level > self.top_level {
            self.top_level = target_level;
        }

        let new_node = Node {
            key: search_key.into(),
            value: value.into(),
            next: vec![None; target_level + 1]
        };

        // Size accounting on add
        self.update_value_size_bytes(0, new_node.size());
        self.length += 1;
        self.arena.push(new_node);

        let new_idx = self.arena.len() - 1;
        for level in 0..=target_level {
            let breadcrumb_idx = update[level];
            self.arena[new_idx].next[level] = self.arena[breadcrumb_idx].next[level];
            self.arena[breadcrumb_idx].next[level] = Some(new_idx);
        }
    }

    fn update_value_size_bytes(&mut self, old_size: usize, new_size: usize) -> () {
        self.size_bytes = self.size_bytes - old_size + new_size;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_put_and_get() {
        let mut list = SkipList::new();
        list.put(b"apple", b"red");
        list.put(b"banana", b"yellow");

        assert_eq!(list.get(b"apple"), Some(&b"red"[..]));
        assert_eq!(list.get(b"banana"), Some(&b"yellow"[..]));
        assert_eq!(list.get(b"grape"), None);
    }

    #[test]
    fn test_node_size_no_next() {

        let node = Node {
            key: b"apple".into(),
            value: b"red".into(),
            next: vec![]
        };

        assert_eq!(node.size(), 8);
    }

    #[test]
    fn test_node_size_3_next() {
        let next_count: usize = 3;

        let node = Node {
            key: b"apple".into(),
            value: b"red".into(),
            next: vec![Some(1); next_count]
        };

        assert_eq!(node.size(), 8 + next_count * size_of::<Option<usize>>());
    }

    #[test]
    fn test_skiplist_size_bounds() {
        let mut list = SkipList::new();
        let initial_size = list.size_bytes;

        list.put(b"apple", b"red");

        let key_val_size = 5 + 3;
        let min_node_size = key_val_size + (1 * size_of::<Option<usize>>());
        let max_node_size = key_val_size + (SL_MAX_LEVEL * size_of::<Option<usize>>());

        assert!(list.size_bytes >= initial_size + min_node_size);
        assert!(list.size_bytes <= initial_size + max_node_size);
    }
}