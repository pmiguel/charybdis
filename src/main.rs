use charybdis::db::Db;
use charybdis::wal;
use charybdis::wal::WalRecord;

fn main() {
    let mut wal = wal::Wal::new();
    wal.init().unwrap();

    // Bootstrap WAL directly
    let r1 = WalRecord::new(b"val1", b"1", 1, 0);
    let r2 = WalRecord::new(b"val2", b"2", 1, 1);
    let r3 = WalRecord::new(b"val3", b"3", 1, 2);
    let r4 = WalRecord::new(b"val4", b"4", 1, 3);

    // Delete val3
    let r5 = WalRecord::new(b"val3", b"", 2, 4);

    // Update val4
    let r6 = WalRecord::new(b"val4", b"44", 1, 5);

    wal.append(&r1).unwrap();
    wal.append(&r2).unwrap();
    wal.append(&r3).unwrap();
    wal.append(&r4).unwrap();
    wal.append(&r5).unwrap();
    wal.append(&r6).unwrap();

    // Verify records
    wal.verify().unwrap();

    // init DB, will load wal
    let mut db = Db::new();
    db.init().unwrap();

    // assertions
    let result1 = String::from_utf8_lossy(db.get(&"val1").unwrap());
    let result2 = String::from_utf8_lossy(db.get(&"val2").unwrap());
    let result3 = String::from_utf8_lossy(db.get(&"val3").unwrap_or(b"<null>"));
    let result4 = String::from_utf8_lossy(db.get(&"val4").unwrap());

    println!("{}", result1);
    println!("{}", result2);
    println!("{}", result3);
    println!("{}", result4);


}