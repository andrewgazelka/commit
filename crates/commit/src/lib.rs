//! # commit
//!
//! Zero-allocation commit tracking for Rust binaries.
//! Pin data to git commits and compare them temporally.
//!
//! ## Usage
//!
//! ```rust,ignore
//! // When serializing: pin data to current commit
//! let data = commit::Pinned::new(MyStruct { field: "value" });
//! serde_json::to_writer(file, &data)?;
//!
//! // When deserializing: check commit and apply schema migrations
//! let data: commit::Pinned<MyStruct> = serde_json::from_reader(file)?;
//! let time = data.commit_time()?;
//!
//! // Define schema version boundaries
//! const V2_IDX: u16 = commit::Time::from_hash_const(&SCHEMA_V2_COMMIT).index();
//! const V3_IDX: u16 = commit::Time::from_hash_const(&SCHEMA_V3_COMMIT).index();
//!
//! // Pattern match on commit time index
//! match time.index() {
//!     ..=V3_IDX => { /* v3+ */ }
//!     ..V2_IDX => { /* v2 */ }
//!     _ => { /* v1 */ }
//! }
//! ```

// Re-export everything from commit-info
pub use commit_info::{Hash, NotFound, Time, commit, dirty};

// Re-export everything from commit-pinned
pub use commit_pinned::{Ignored, Pinned};
