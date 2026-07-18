use std::io::Cursor;

struct BloomFilter {
    bitmap: Vec<u8>,
    k: u8
}

impl BloomFilter {
    pub fn new(expected_keys: usize) -> Self {
        // TODO consider using user-configurable bits-per-key (BPK) and calculate K for 1% false positives
        let total_bits = expected_keys * 10; // 10 bits per key
        let total_bytes = std::cmp::max(1, (total_bits + 7) / 8);

        BloomFilter {
            bitmap: vec![0; total_bytes],
            k: 7,
        }
    }

    pub fn add(&mut self, key: &[u8]) {
        let bit_count =  self.bitmap.len() * 8;
        let (h1, h2) = Self::split_hash(key);

        for i in 0..self.k {
            // hc = h1 + (i * h2)
            let combined = h1.wrapping_add((i as u64).wrapping_mul(h2));

            let bit_idx = (combined % (bit_count as u64)) as usize;
            let byte_idx = bit_idx / 8;
            let bit_offset = bit_idx % 8;

            self.bitmap[byte_idx] |= 1 << (bit_offset)
        }
    }

    pub fn may_contain(&self, key: &[u8]) -> bool {
        let bit_count =  self.bitmap.len() * 8;
        let (h1, h2) = Self::split_hash(key);

        for i in 0..self.k {
            // Double hashing formula = hc = h1 + (i * h2)
            let combined = h1.wrapping_add((i as u64).wrapping_mul(h2));

            let bit_idx = (combined % (bit_count as u64)) as usize;
            let byte_idx = bit_idx / 8;
            let bit_offset = bit_idx % 8;

            let is_set = (self.bitmap[byte_idx] & (1 << bit_offset)) != 0;
            if !is_set {
                return false
            }
        }
        true
    }

    fn split_hash(key: &[u8]) -> (u64, u64) {
        let base = murmur3::murmur3_x64_128(&mut Cursor::new(key), 0).expect("Could not hash the specified key");
        let h1 = (base >> 64) as u64;
        let h2 = (base & 0xFFFFFFFFFFFFFFFF) as u64;

        (h1, h2)
    }
}