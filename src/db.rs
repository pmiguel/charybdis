use std::io;
use crate::memtable::MemTable;
use crate::wal::{Wal, WalRecord};

const MAX_MEMTABLE_SIZE: usize = 1024;

pub struct Db {
    wal: Wal,
    active_mem_table: MemTable,
    flushing_mem_table: Option<MemTable>,
}

impl Db {
    pub fn new() -> Db {
        Db {
            wal: Wal::new(),
            active_mem_table: MemTable::new(),
            flushing_mem_table: None
        }
    }

    pub fn init(&mut self) -> Result<(), io::Error> {
        println!("DB::starting init...");
        self.wal.init()?;
        let records = self.wal.recover()?;
        println!("DB::restoring memtable state...");
        for record in records {
            let op_result;
            match record.record_type {
                1 => op_result = self.active_mem_table.put(&record.key, &record.val),
                2 => op_result = self.active_mem_table.del(&record.key),
                _ => continue,
            }
            match op_result {
                Ok(()) => continue,
                // TODO handle memtable errors
                Err(e) => {
                    println!("DB::Unrecoverable error while initializing. {}", e);
                    panic!("{}", e)
                }
            }
        }
        println!("DB::<READY>...");
        Ok(())
    }

    pub fn put<K, V>(&mut self, key: K, val: V) -> Result<(), io::Error>
    where
        K: AsRef<[u8]>,
        V: AsRef<[u8]>,
    {
        println!("DB::putting key...");
        let k_slice = key.as_ref();
        let v_slice = val.as_ref();
        let record = WalRecord::new(k_slice, v_slice, 1, 0);

        self.wal.append(&record)?;
        match self.active_mem_table.put(k_slice, v_slice) {
            Ok(()) => {
                if self.active_mem_table.size() >= MAX_MEMTABLE_SIZE {
                    println!("DB::active memtable crossed threshold. Initiating memtable rotation...");
                    if self.flushing_mem_table.is_some() {
                        println!("DB::a pending memtable is still flushing... Write Stall.");
                        panic!("Write stall! Disk is too slow!");
                    }
                    println!("DB::Freezing memtable...");
                    self.active_mem_table.freeze();

                    println!("DB::rotating memtable. Marking old for flushing...");
                    self.flushing_mem_table = Some(std::mem::replace(&mut self.active_mem_table, MemTable::new()));
                    // TODO flush memtable to disk SSTables and rotate WAL
                }
                Ok(())
            },
            // TODO handle memtable errors
            Err(_) => Err(io::Error::new(io::ErrorKind::Other, "An error has ocurred"))
        }
    }

    pub fn delete<K>(&mut self, key: K) -> Result<(), io::Error>
    where
        K: AsRef<[u8]>,
    {
        println!("DB::deleting key...");
        let k_slice = key.as_ref();
        let record = WalRecord::new(k_slice, &[], 2, 0);

        self.wal.append(&record)?;
        match self.active_mem_table.del(k_slice) {
            Ok(()) => Ok(()),
            // TODO handle memtable errors
            Err(_) => Err(io::Error::new(io::ErrorKind::Other, "An error has ocurred"))
        }
    }

    pub fn get<'a, K, V>(&'a self, key: &K) -> Option<V>
    where
        K: AsRef<[u8]>,
        V: From<&'a [u8]>,
    {
        println!("DB::getting key...");
        let k_slice = key.as_ref();
        self.active_mem_table.get(k_slice).map(V::from)
    }
}