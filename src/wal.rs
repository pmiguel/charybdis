use std::{fs, io};
use std::io::Write;

const WAL_FILE_PATH: &str = "./odb.wal";

pub struct Wal {
    curr_file: Option<fs::File>,
    curr_seq_no: u64
}

pub struct WalRecord {
    record_type: u8,
    batch_id: u64,
    key_len: u32,
    key: Vec<u8>,
    val_len: u32,
    val: Vec<u8>
}

impl WalRecord {
    pub fn new(key: &[u8], val: &[u8], record_type: u8) -> WalRecord {
        let key_len = (key.len() as u32).to_le_bytes();
        let val_len = (val.len() as u32).to_le_bytes();

        WalRecord {
            record_type,
            batch_id: 0x0,
            key_len: key.len() as u32,
            val_len: val.len() as u32,
            key: key.into(),
            val: val.into()
        }
    }

    pub fn pack(&self) -> Vec<u8> {
        let key_len = (self.key.len() as u32).to_le_bytes();
        let val_len = (self.val.len() as u32).to_le_bytes();
        let total_len =
            size_of::<u32>() +  // CRC32
            size_of::<u8>() +   // record_type
            size_of::<u64>() +  // batch_id
            size_of::<u32>() +  // key_len (le)
            size_of::<u32>() +  // val_len (le)
            self.key.len() +    // key
            self.val.len();     // val

        let mut buf = Vec::with_capacity(total_len);

        buf.extend_from_slice(&[0,0,0,0]); // CRC32 placeholder
        buf.push(self.record_type);
        buf.extend_from_slice(&vec![0; size_of::<u64>()]); // 0x0 batch_id
        buf.extend_from_slice(&key_len[..]);
        buf.extend_from_slice(&val_len[..]);
        buf.extend_from_slice(&self.key);
        buf.extend_from_slice(&self.val);

        let crc32 = crc32fast::hash(&buf[4..]);
        buf[0..4].copy_from_slice(&crc32.to_le_bytes());

        buf
    }
}

impl Wal {
    pub fn new() -> Wal {
        Wal {
            curr_file: None,
            curr_seq_no: 0
        }
    }

    pub fn init(&mut self) -> Result<(), std::io::Error> {
        let mut open_options = fs::OpenOptions::new();
        open_options
            .append(true)
            .create(true);

        match open_options.open(WAL_FILE_PATH) {
            Ok(file) => {
                self.curr_file = Some(file);
                Ok(())
            }
            Err(e) => { Err(e) }
        }
    }

    pub fn append(&mut self, record: &WalRecord) -> Result<(), io::Error> {
        let buf = record.pack();

        let file = self.curr_file.as_mut().ok_or_else(|| {
            io::Error::new(io::ErrorKind::NotConnected, "WAL file not initialized")
        })?;

        file.write_all(&buf)?;
        file.sync_data()?;

        Ok(())
    }
}