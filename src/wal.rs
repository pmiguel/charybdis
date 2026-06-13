use std::{fmt, fs, io};
use std::io::{Read, Seek, Write};
use bytes;
use bytes::Buf;

const WAL_FILE_PATH: &str = "./odb.wal";

pub struct Wal {
    curr_file: Option<fs::File>,
    // curr_seq_no: u64 TODO sequence number
}

pub struct WalRecord {
    pub record_type: u8,
    // batch_id: u64, TODO
    seq_no: u64,
    pub key: Vec<u8>,
    pub val: Vec<u8>
}

impl fmt::Display for WalRecord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(walrecord: k:{}, v:{})", String::from_utf8_lossy(&self.key), String::from_utf8_lossy(&self.val))
    }
}

impl WalRecord {
    pub fn new(key: &[u8], val: &[u8], record_type: u8, seq_no: u64) -> WalRecord {
        WalRecord {
            record_type,
            // batch_id: 0x0, TODO
            seq_no,
            key: key.into(),
            val: val.into()
        }
    }

    pub fn header_size() -> usize {
            size_of::<u32>() +  // CRC32
            size_of::<u8>() +   // record_type
            size_of::<u64>() +  // batch_id
            size_of::<u64>() +  // seq_no
            size_of::<u32>() +  // key_len (le)
            size_of::<u32>()  // val_len (le)
    }

    pub fn pack(&self) -> Vec<u8> {
        let key_len = (self.key.len() as u32).to_le_bytes();
        let val_len = (self.val.len() as u32).to_le_bytes();
        let total_len = WalRecord::header_size() + self.key.len() +  self.val.len();

        let mut buf = Vec::with_capacity(total_len);

        buf.extend_from_slice(&[0,0,0,0]);                 // CRC32 placeholder
        buf.push(self.record_type);                              // record type
        buf.extend_from_slice(&vec![0; size_of::<u64>()]); // 0x0 batch_id
        buf.extend_from_slice(&self.seq_no.to_le_bytes());       // 0x0 seq_no
        buf.extend_from_slice(&key_len[..]);               // key length
        buf.extend_from_slice(&val_len[..]);               // val length
        buf.extend_from_slice(&self.key);                        // key
        buf.extend_from_slice(&self.val);                        // val

        let crc32 = crc32fast::hash(&buf[4..]);
        buf[0..4].copy_from_slice(&crc32.to_le_bytes());

        buf
    }
}

impl Wal {
    pub fn new() -> Wal {
        Wal {
            curr_file: None,
            // curr_seq_no: 0 TODO sequence number
        }
    }

    pub fn init(&mut self) -> Result<(), io::Error> {
        let mut open_options = fs::OpenOptions::new();
        open_options
            .read(true)
            .write(true)
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

    pub fn recover(&mut self) -> Result<Vec<WalRecord>, io::Error> {
        let file = self.curr_file.as_mut().ok_or_else(|| {
            io::Error::new(io::ErrorKind::NotConnected, "WAL file not initialized")
        })?;

        let mut record_buf = vec![];


        file.rewind()?;

        let mut buf = vec![];
        let _ = file.read_to_end(&mut buf)?;
        let mut ptr = &buf[..];

        // As long as there are bytes left in our pointer...
        while ptr.has_remaining() {
            // Save a copy of the pointer BEFORE we read the CRC
            // so we can calculate the payload size later for the CRC check.
            let record_start_len = ptr.len();

            // Reading automatically advances `ptr`! No offsets needed!
            let record_crc32 = ptr.get_u32_le();
            let record_type   = ptr.get_u8();
            let _                 = ptr.get_u64_le(); // TODO batch_id
            let seq_no       = ptr.get_u64_le();
            let kl           = ptr.get_u32_le();
            let vl           = ptr.get_u32_le();

            // Extract Key
            let key = &ptr[..kl as usize];
            ptr.advance(kl as usize); // Move the pointer past the key

            // Extract Value
            let val = &ptr[..vl as usize];
            ptr.advance(vl as usize); // Move the pointer past the value

            // --- CRC Check ---
            // How many bytes did we just consume for the payload?
            let payload_len = (record_start_len - ptr.len()) - 4; // Subtract the 4 bytes of CRC

            // Grab the exact payload bytes from our original buffer
            let base_offset = buf.len() - record_start_len;
            let payload_bytes = &buf[base_offset + 4 .. base_offset + 4 + payload_len];

            let calc_crc32 = crc32fast::hash(payload_bytes);

            if calc_crc32 == record_crc32 {
                record_buf.push(WalRecord::new(key, val, record_type, seq_no));
            }
        }

        Ok(record_buf)
    }

    pub fn inspect(&mut self) -> Result<(), io::Error> {
        let records = self.recover()?;
        let _: () = records.iter().map(|r| println!("{}", r)).collect();
        Ok(())
    }

    pub fn verify(&mut self) -> Result<(), io::Error> {
        println!("== Write-Ahead Log Inspection ==");
        let file = self.curr_file.as_mut().ok_or_else(|| {
            io::Error::new(io::ErrorKind::NotConnected, "WAL file not initialized")
        })?;

        file.rewind()?;

        let mut buf = vec![];
        let bytes_read = file.read_to_end(&mut buf)?;
        println!("-> wal file read: {} bytes", bytes_read);

        let mut ptr = &buf[..];

        while ptr.has_remaining() {
            let record_start_len = ptr.len();

            let record_crc32 = ptr.get_u32_le();
            let record_type  = ptr.get_u8();
            let batch_id     = ptr.get_u64_le();
            let seq_no       = ptr.get_u64_le();
            let kl           = ptr.get_u32_le();
            let vl           = ptr.get_u32_le();

            let key = &ptr[..kl as usize];
            ptr.advance(kl as usize); // Move the pointer past the key

            let val = &ptr[..vl as usize];
            ptr.advance(vl as usize); // Move the pointer past the value

            let payload_len = (record_start_len - ptr.len()) - 4; // Subtract the 4 bytes of CRC

            let base_offset = buf.len() - record_start_len;
            let payload_bytes = &buf[base_offset + 4 .. base_offset + 4 + payload_len];

            let calculated_crc32 = crc32fast::hash(payload_bytes);

            println!("h[crc:{},crc_check:{},t:{},b:{},seq:{},kl:{},vl:{}]\tb[k:{},v:{}]",
                     record_crc32,
                     calculated_crc32 == record_crc32,
                     record_type,
                     batch_id,
                     seq_no,
                     kl,
                     vl,
                     String::from_utf8_lossy(key), // Safer than unwrap() if bytes aren't valid UTF8
                     String::from_utf8_lossy(val),
            );
        }

        Ok(())
    }
}