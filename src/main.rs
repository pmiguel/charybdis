use onyxdb::memtable;

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
}