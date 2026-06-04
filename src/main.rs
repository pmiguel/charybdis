use onyxdb::memtable;
use onyxdb::wal;
fn main() {

    let mut memtable = memtable::MemTable::new();

    memtable.put(b"Key", b"value");

    let read  = memtable.get(b"Key");

    match read {
        Some(val) => {
            println!("{}", str::from_utf8(val).unwrap());
        }
        None => {
            println!("No value");
        }
    }

    let mut wal = wal::Wal::new();
    wal.init().unwrap();

    match wal.append(b"key", b"HELLOWORLD") {
        Ok(()) => {
            println!("WAL append successful");
        }
        Err(e) => {
            println!("{}", e)
        }
    }
}