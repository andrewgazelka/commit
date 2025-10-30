# commit

Zero-allocation commit tracking for Rust binaries. Pin data to git commits and compare them temporally.

## Why

When you serialize data, you need to know which version of your code produced it. Git commits are the source of truth for code versions. This embeds commit info directly into your types at compile time.

Schema evolution: deserialize old data, check its commit, apply migrations based on temporal ordering.

## Crates

- `commit`: Unified interface (re-exports `commit-info` and `commit-pinned`)
- `commit-info`: Core commit tracking with zero-allocation history lookups via `include_bytes!`
- `commit-pinned`: Wrapper type that pins any `T` to a commit with full serde support

## Usage

```rust
// When serializing: pin data to current commit
let data = commit::Pinned::new(MyStruct { field: "value" });
serde_json::to_writer(file, &data)?;

// When deserializing: check which schema version
let data: commit::Pinned = serde_json::from_reader(file)?;
let time = data.commit_time()?;

// Define schema version boundaries
const V2_IDX: u16 = commit::Time::from_hash_const(&SCHEMA_V2_COMMIT).index();
const V3_IDX: u16 = commit::Time::from_hash_const(&SCHEMA_V3_COMMIT).index();

// Determine which schema version
let version = match time.index() {
    ..=V3_IDX => "v3",
    ..V2_IDX => "v2",
    _ => "v1",
};

println!("Data from schema version: {}", version);
```

Commit hashes are SHA-256 of git commit IDs. History embedded as raw bytes at compile time.
