use std::{fs, io};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use bytes::Buf;

const MAX_BLOCK_SIZE: usize = 4 * 1024; // 4 KiB

struct SSTableBuilder {
    file_name: String,
    curr_file: Option<File>,
    curr_block: Block,
    curr_offset: u64,
    index_entries: Vec<IndexEntry>,
    last_key: Vec<u8>,
    finished: bool
}

#[cfg(unix)]
fn read_at(file: &File, buf: &mut [u8], offset: u64) -> io::Result<()> {
    use std::os::unix::fs::FileExt;
    file.read_exact_at(buf, offset);
    Ok(())
}

#[cfg(windows)]
fn read_at(file: &File, buf: &mut [u8], offset: u64) -> io::Result<()> {
    use std::os::windows::fs::FileExt;
    file.seek_read(buf, offset)?;
    Ok(())
}

struct SSTableReader {
    curr_file: Option<File>,
    index_start_offset: u64,
    index_entries: Vec<IndexEntry>,
}

impl SSTableBuilder {
    fn new(file_name: String) -> SSTableBuilder {
        SSTableBuilder {
            file_name,
            curr_file: None,
            curr_block: Block::new(),
            curr_offset: 0,
            index_entries: Vec::new(),
            last_key: vec![],
            finished: false
        }
    }

    pub fn append(&mut self, key: &[u8], value: &[u8]) -> Result<(), std::io::Error> {
        if self.finished {
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "SSTable already finished"));
        }
        self.ensure_file()?;
        self.curr_block.append(key, value);
        self.last_key = key.to_vec();

         if self.curr_block.size() >= MAX_BLOCK_SIZE {
             let sst_file = self.curr_file.as_mut().unwrap();
             let block_bytes = self.curr_block.as_vec();

             sst_file.write_all(block_bytes.as_slice())?;

             self.index_entries.push(IndexEntry::new(self.last_key.clone(), self.curr_offset));

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

                match open_options.open(self.file_name.clone()) {
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
        if self.finished {
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "SSTable already finished"));
        }
        self.ensure_file()?;

        // Close out SSTable, flush in-memory block even if it didn't reach MAX_BLOCK_SIZE
        if self.curr_block.payload.len() > 0 {
            let sst_file = self.curr_file.as_mut().unwrap();
            let block_bytes = self.curr_block.as_vec();

            sst_file.write_all(block_bytes.as_slice())?;

            self.index_entries.push(IndexEntry::new(self.last_key.clone(), self.curr_offset));

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
        self.finished = true;
        Ok(())
    }
}

impl SSTableReader {
    pub fn open(&self, path: &str) -> Result<Self, std::io::Error> {
        let mut file = File::open(path)?;
        let file_size = file.metadata()?.len();

        // Get Index block range
        let mut footer_start_offset_buf = [0u8; 8];
        file.seek(SeekFrom::End(-8))?;
        file.read_exact(&mut footer_start_offset_buf)?;

        let index_start_offset = u64::from_le_bytes(footer_start_offset_buf);
        let footer_start_offset = file_size - 8;
        let index_size = footer_start_offset - index_start_offset;

        // Read Index block
        let mut index_buff = vec![0u8; index_size as usize];
        file.seek(SeekFrom::Start(index_start_offset))?;
        file.read_exact(&mut index_buff)?;

        // Parse Index block into IndexEntries
        let mut index_entries = vec![];
        let mut ptr = &index_buff[..];

        while ptr.has_remaining() {
            let key_length = ptr.get_u32_le();
            let key = &ptr[..key_length as usize];
            ptr.advance(key_length as usize);
            let last_offset = ptr.get_u64_le();
            index_entries.push(IndexEntry::new(key.into(), last_offset));
        }

        Ok(SSTableReader {
            curr_file: Some(file),
            index_entries,
            index_start_offset
        })
    }

    pub fn get(&mut self, search_key: Vec<u8>) -> Result<Option<Vec<u8>>, std::io::Error> {
        let index_entries_length = self.index_entries.len();
        if index_entries_length == 0 {
            return Ok(None);
        }

        // Find index entry matching out search key
        let mut found_index_entry: usize = 0;
        let mut found = false;
        for i in (0..index_entries_length) {
            let entry = &self.index_entries[i];
            if entry.last_key >= search_key {
                found = true;
                found_index_entry = i;
                break;
            }
        }

        // Key is higher than the stored indexes. Not on this file.
        if !found {
            return Ok(None);
        }

        // Fetch target block offset and calculate it's size
        let target_block_offset = (&self.index_entries[found_index_entry]).block_offset;
        let target_block_size = if found_index_entry == index_entries_length - 1 {
            self.index_start_offset - target_block_offset
        } else {
            &self.index_entries[found_index_entry + 1].block_offset - target_block_offset
        };

        // Read block from the sst
        let sst = match self.curr_file.as_mut() {
            None => {
                return Err(io::Error::new(io::ErrorKind::Other, "SST File not assigned"));
            },
            Some(f) => f,
        };

        let mut block_buff = vec![0u8; target_block_size as usize];
        read_at(sst, &mut block_buff, target_block_offset)?;

        let block_data_size = block_buff.len() - 5;
        let mut ptr = &block_buff[..block_data_size];

        // TODO CRC32 block check

        while ptr.has_remaining() {
            let key_length = ptr.get_u32_le();
            let key = &ptr[..key_length as usize];
            ptr.advance(key_length as usize);
            let value_length = ptr.get_u32_le();
            let value = &ptr[..value_length as usize];
            ptr.advance(value_length as usize);
            if key == search_key {
                return Ok(Some(value.into()));
            }

            // No point in keep looking, it's a sorted block
            if key > search_key.as_slice() {
                break;
            }
        }
        Ok(None)
    }
}

struct IndexEntry {
    last_key: Vec<u8>,
    block_offset: u64
}

impl IndexEntry {
    pub fn new(last_key: Vec<u8>, block_offset: u64) -> IndexEntry {
        IndexEntry {
            last_key,
            block_offset
        }
    }

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