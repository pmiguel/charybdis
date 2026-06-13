use std::fs::File;

struct SSTableBuilder {
    file: File,
    curr_block: Block,
    curr_offset: u64,
    index_entries: Vec<IndexEntry>
}

struct IndexEntry {
    last_key: Vec<u8>,
    block_offset: u64
}

struct Block {
    payload: Vec<u8>,
    compression: u8,
    crc32: u32
}

impl Block {
    fn new() -> Block {
        Block {
            payload: vec![],
            compression: 0x0,
            crc32: 0x0
        }
    }

    pub fn append(&mut self, key: &[u8], val: &[u8]) -> () {
        let key_len = key.len() as u32;
        let val_len = val.len() as u32;

        // Append to existing payload buffer
        self.payload.extend_from_slice(&key_len.to_le_bytes());
        self.payload.extend_from_slice(key);
        self.payload.extend_from_slice(&val_len.to_le_bytes());
        self.payload.extend_from_slice(val);
    }

    pub fn size(&self) -> usize {
        self.payload.len() * size_of::<u8>() + size_of::<u8>() + size_of::<u32>()
    }

    // For flushing/serialization
    // Calculates final block crc32
    pub fn as_vec(&self) -> Vec<u8> {
        let mut out_buf: Vec<u8> = Vec::with_capacity(self.size());
        out_buf.extend_from_slice(self.payload.as_slice());
        out_buf.push(self.compression);
        let crc32 = crc32fast::hash(out_buf.as_slice());
        out_buf.extend_from_slice(&crc32.to_le_bytes());
        out_buf
    }
}