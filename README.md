# commit

Zero-allocation commit tracking for Rust binaries. Pin data to git commits and compare them temporally.

## Why

When you serialize data, you need to know which version of your code produced it. Git commits are the source of truth for code versions. This embeds commit info directly into your types at compile time.

Schema evolution: deserialize old data, check its commit, apply migrations based on temporal ordering.

## Crates

- `commit-info`: Core commit tracking with zero-allocation history lookups via `include_bytes!`
- `commit-pinned`: Wrapper type that pins any `T` to a commit with full serde support

## Usage

```rust
use commit_pinned::CommitPinned;
use commit_info::CommitTime;

// When serializing: pin data to current commit
let data = CommitPinned::new(MyStruct { field: "value" });
serde_json::to_writer(file, &data)?;

// When deserializing: check commit and apply schema migrations
let data: CommitPinned<MyStruct> = serde_json::from_reader(file)?;
let time = data.commit_time()?;

// Define schema version boundaries by their indices
const V2_IDX: u16 = CommitTime::from_hash_const(&SCHEMA_V2_COMMIT).index();
const V3_IDX: u16 = CommitTime::from_hash_const(&SCHEMA_V3_COMMIT).index();

// Pattern match on commit time index with range patterns
let migrated = match time.index() {
    ..=V3_IDX => {
        // Schema v3+: added middle name (newer = lower index)
        data.into_content()
    }
    ..V2_IDX => {
        // Schema v2: split into first/last
        let v2: V2Struct = data.into_content();
        V3Struct { first: v2.first, last: v2.last, middle: None }
    }
    _ => {
        // Schema v1: single "name" field
        let v1: V1Struct = serde_json::from_value(/* ... */)?;
        V3Struct { first: v1.name, last: "", middle: None }
    }
};
```

Commit hashes are SHA-256 of git commit IDs. History embedded as raw bytes at compile time.
