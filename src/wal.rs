use std::{fs, io};
use std::io::Write;

const WAL_FILE_PATH: &str = "./odb.wal";

struct Wal {
    curr_file: Option<std::fs::File>,
}

impl Wal {
    pub fn new() -> Wal {
        Wal {
            curr_file: None,
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

    pub fn append(&mut self, key: &[u8], val: &[u8]) -> Result<(), io::Error> {
        let key_len = (key.len() as u32).to_le_bytes();
        let val_len = (val.len() as u32).to_le_bytes();
        let total_len = size_of::<u32>() * 2 + key.len() + val.len();

        let mut buf = Vec::with_capacity(total_len);

        buf.extend_from_slice(&key_len[..]);
        buf.extend_from_slice(key);
        buf.extend_from_slice(&val_len[..]);
        buf.extend_from_slice(val);

        let file = self.curr_file.as_mut().ok_or_else(|| {
            io::Error::new(io::ErrorKind::NotConnected, "WAL file not initialized")
        })?;

        file.write_all(&buf)?;
        file.sync_data()?;

        Ok(())
    }
}