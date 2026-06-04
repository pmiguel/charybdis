const MAX_LEVEL: usize = 16;

pub struct Node {
    key: Vec<u8>,
    value: Vec<u8>,
    next: Vec<Option<usize>>
}


pub struct SkipList {
    arena: Vec<Node>,
    head: usize,
    max_level: usize,
}

impl SkipList {
    pub fn new() -> Self {
        let head = Node{
            key: vec![],
            value: vec![],
            next: vec![None; MAX_LEVEL]
        };

        SkipList {
            arena: vec![head],
            head: 0,
            max_level: 0,
        }
    }

    pub fn get(&self, search_key: &[u8]) -> Option<&[u8]> {
        let mut current_idx = self.head;
        for level in (0..=self.max_level).rev() {
            loop {
                let next_pointer = self.arena[current_idx].next[level];
                match next_pointer {
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
        while rand::random::<bool>() && level < MAX_LEVEL - 1 {
            level += 1;
        }
        level
    }

    pub fn put(&mut self, search_key: &[u8], value: &[u8]) -> () {
        let mut update = vec![self.head; MAX_LEVEL];
        let mut current_idx = self.head;

        for level in (0..=self.max_level).rev() {
            loop {
                let next_pointer = self.arena[current_idx].next[level];
                match next_pointer {
                    None => {
                        update[level] = current_idx;
                        break;
                    }
                    Some(next_idx) => {
                        // equivalent to &self.arena[next_idx].key[..]
                        let next_key = self.arena[next_idx].key.as_slice();
                        if next_key == search_key {
                            self.arena[next_idx].value = value.into();
                            return
                        } else if next_key < search_key {
                            current_idx = next_idx;
                        } else {
                            update[level] = current_idx;
                            break;
                        }
                    }
                }
            }
        }
        let target_level = Self::flip();
        if target_level > self.max_level {
            self.max_level = target_level;
        }

        let new_node = Node {
            key: search_key.into(),
            value: value.into(),
            next: vec![None; target_level + 1]
        };
        self.arena.push(new_node);
        let new_idx = self.arena.len() - 1;
        for level in 0..=target_level {
            // 1. Get the breadcrumb node index for this level
            let breadcrumb_idx = update[level];

            // 2. The new node points to whatever the breadcrumb WAS pointing to
            self.arena[new_idx].next[level] = self.arena[breadcrumb_idx].next[level];

            // 3. The breadcrumb now points to the new node
            self.arena[breadcrumb_idx].next[level] = Some(new_idx);
        }
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
}