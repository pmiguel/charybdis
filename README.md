# OnyxDB 🪨

> A fast, embedded LSM-Tree Key-Value store written entirely in Rust.

**OnyxDB** is a lightweight, high-performance storage engine modeled after systems like LevelDB, RocksDB, and Pebble. It is built from scratch as an exercise in deep systems engineering, focusing on memory layout, lock-free concurrency, and bypassing operating system I/O bottlenecks.

## Architecture

OnyxDB uses a **Log-Structured Merge-tree (LSM-tree)** architecture, optimizing for blisteringly fast sequential write throughput and immutability.

The storage engine is broken down into four core mechanical layers:

### 1. The MemTable (Memory Table)
An in-memory **Arena-backed SkipList**. By using a vector-backed arena instead of standard pointers (`Box`/`Rc`), OnyxDB ensures aggressive CPU cache locality and completely bypasses Rust's borrow-checker bottlenecks. It yields `O(log n)` read and write performance with zero heap allocations on the read path.

### 2. The WAL (Write-Ahead Log)
To guarantee durability, all writes are appended to a binary log file and physically `fsync`'d to non-volatile storage before the MemTable is updated. The WAL uses CRC32 checksums to detect and recover from torn writes in the event of a power failure.

### 3. SSTables (Sorted String Tables)
When the MemTable reaches its capacity, it is frozen and sequentially flushed to disk as an immutable SSTable. Files are broken into 4KB hardware-aligned blocks and feature internal index blocks and Bloom Filters to minimize disk reads.

### 4. Background Compaction
A background state machine continuously merges overlapping SSTables (Level 0 through Level N), dropping tombstoned (deleted) keys and reclaiming disk space using atomic file rename operations (Two-Phase Commits via a `MANIFEST` file).

## 🚀 Quick Start (Draft API)

```rust
use onyxdb::Db;

fn main() -> onyxdb::Result<()> {
    // Open or create the database
    let mut db = Db::open("./onyx_data")?;

    // Fast, in-memory updates backed by WAL durability
    db.put(b"database", b"onyx")?;
    db.put(b"architecture", b"lsm-tree")?;

    // Zero-copy read path returning Option<&[u8]>
    if let Some(value) = db.get(b"database") {
        println!("Found: {:?}", std::str::from_utf8(value).unwrap());
    }

    Ok(())
}
```

## Roadmap

- [x] **Phase 1: Memory Layout** - Vector-backed SkipList implementation.
- [x] **Phase 2: In-Memory Engine** - `O(log n)` `put` and `get` operations.
- [ ] **Phase 3: Durability** - Binary serialization and WAL `fsync` implementation.
- [ ] **Phase 4: Disk Archives** - Flushing MemTables to immutable SSTables.
- [ ] **Phase 5: Read Path** - Integrating MemTable and SSTable lookups.
- [ ] **Phase 6: Compaction** - Background SSTable merging and garbage collection.

## Tech Stack & Dependencies
OnyxDB intentionally avoids heavy external database frameworks. It relies strictly on standard Rust systems utilities:
* `rand`: Probabilistic distributions for SkipList levels.
