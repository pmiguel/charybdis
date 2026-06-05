use onyxdb::memtable;
use onyxdb::wal;
use onyxdb::wal::WalRecord;

fn main() {

    // let mut memtable = memtable::MemTable::new();
    //
    // memtable.put(b"Key", b"value");
    //
    // let read  = memtable.get(b"Key");
    //
    // match read {
    //     Some(val) => {
    //         println!("{}", str::from_utf8(val).unwrap());
    //     }
    //     None => {
    //         println!("No value");
    //     }
    // }

    let mut wal = wal::Wal::new();
    wal.init().unwrap();

    // for idx in 0..10 {
    //     let temp_rec = WalRecord::new(b"DEADBEEF", b"XXXX", 1, idx);
    //     match wal.append(&temp_rec) {
    //         Ok(()) => {
    //             println!("WAL append successful");
    //         }
    //         Err(e) => {
    //             println!("{}", e)
    //         }
    //     }
    //
    // }

    wal.inspect().unwrap();
}