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
}

impl SkipList {
    pub fn new() -> Self {
        let head = Node{
            key: vec![],
            value: vec![],
            next: vec![None; SL_MAX_LEVEL]
        };

        SkipList {
            arena: vec![head],
            head_idx: 0,
            top_level: 0,
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

        self.arena.push(Node {
            key: search_key.into(),
            value: value.into(),
            next: vec![None; target_level + 1]
        });

        let new_idx = self.arena.len() - 1;
        for level in 0..=target_level {
            let breadcrumb_idx = update[level];
            self.arena[new_idx].next[level] = self.arena[breadcrumb_idx].next[level];
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