use std::fs;
use std::fs::File;
use std::io::Write;

const MAX_BLOCK_SIZE: usize = 4 * 1024; // 4 KiB

struct SSTableBuilder {
    curr_file: Option<File>,
    curr_file_index: u32,
    curr_block: Block,
    curr_offset: u64,
    index_entries: Vec<IndexEntry>,
    last_key: Vec<u8>
}

impl SSTableBuilder {
    fn new() -> SSTableBuilder {
        SSTableBuilder {
            curr_file: None,
            curr_file_index: 0,
            curr_block: Block::new(),
            curr_offset: 0,
            index_entries: Vec::new(),
            last_key: vec![]
        }
    }

    pub fn append(&mut self, key: &[u8], value: &[u8]) -> Result<(), std::io::Error> {
        self.ensure_file()?;
        self.curr_block.append(key, value);
        self.last_key = key.to_vec();

         if self.curr_block.size() >= MAX_BLOCK_SIZE {
             let file = self.curr_file.as_mut().unwrap();
             let block_bytes = self.curr_block.as_vec();

             file.write_all(block_bytes.as_slice())?;

             self.index_entries.push(IndexEntry {
                 block_offset: self.curr_offset,
                 last_key: self.last_key.clone(),
             });

             self.curr_offset = self.curr_offset + block_bytes.len() as u64;
             self.curr_block = Block::new();
        }
        Ok(())
    }

    fn ensure_file(&mut self) -> Result<(), std::io::Error> {
        match self.curr_file {
            Some(_) => { Ok(()) },
            None => {
                let mut open_options = fs::OpenOptions::new();
                open_options.write(true)
                    .append(true)
                    .create(true);

                match open_options.open("file.sst") {
                    Ok(file) => {
                        self.curr_file = Some(file);
                        Ok(())
                    }
                    Err(e) => { Err(e) }
                }
            }
        }
    }

    pub fn finish(&mut self) -> Result<(), std::io::Error> {
        self.ensure_file()?;

        // Close out SSTable, flush in-memory block even if it didn't reach MAX_BLOCK_SIZE
        if self.curr_block.payload.len() > 0 {
            let file = self.curr_file.as_mut().unwrap();
            let block_bytes = self.curr_block.as_vec();

            file.write_all(block_bytes.as_slice())?;

            self.index_entries.push(IndexEntry {
                block_offset: self.curr_offset,
                last_key: self.last_key.clone(),
            });

            self.curr_offset = self.curr_offset + block_bytes.len() as u64;
        }
        let mut foot_buf: Vec<u8> = vec![];
        for index_entry in &self.index_entries {
            foot_buf.extend_from_slice(index_entry.as_vec().as_slice());
        }
        let index_start_offset = self.curr_offset;
        foot_buf.extend_from_slice(&index_start_offset.to_le_bytes());

        let file = self.curr_file.as_mut().unwrap();

        file.write_all(&foot_buf)?;
        file.sync_all()?;

        Ok(())
    }
}

struct IndexEntry {
    last_key: Vec<u8>,
    block_offset: u64
}

impl IndexEntry {
    pub fn as_vec(&self) -> Vec<u8> {
        let key_len = self.last_key.len();
        let buf_size = size_of::<u32>() + key_len + size_of::<u64>();
        let mut buf: Vec<u8> = Vec::with_capacity(buf_size);

        buf.extend_from_slice(&(key_len as u32).to_le_bytes());
        buf.extend_from_slice(&self.last_key);
        buf.extend_from_slice(&self.block_offset.to_le_bytes());

        buf
    }
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