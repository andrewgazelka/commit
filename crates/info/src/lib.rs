// Copyright (c) 2025 Andrew Gazelka
// SPDX-License-Identifier: MIT

use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

/// A 32-byte SHA-256 hash representing a commit
pub type Hash = [u8; 32];

/// Error when a commit hash is not found in the history
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NotFound {
    pub hash: Hash,
}

impl std::fmt::Display for NotFound {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "commit not found in history: {}",
            self.hash
                .iter()
                .map(|b| format!("{:02x}", b))
                .collect::<String>()
        )
    }
}

impl std::error::Error for NotFound {}

/// The current commit hash as a hex string
pub const COMMIT_STRING: &str = env!("GIT_COMMIT");

const DIRTY: &str = env!("GIT_DIRTY");
const HISTORY_LEN: usize = const_str::parse!(env!("GIT_HISTORY_LEN"), usize);

// Include the raw history bytes at compile time
const HISTORY_BYTES: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/history.bin"));

/// The current commit hash as a byte array
pub const COMMIT: Hash = parse_commit_hash(COMMIT_STRING);

/// A timestamp index for a commit, allowing temporal ordering based on commit history.
/// Lower values indicate earlier commits in the repository history.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Time(u16);

impl Time {
    /// Create a Time from a commit hash by looking up its position in history
    pub fn from_hash(hash: &Hash) -> Result<Self, NotFound> {
        get_index(hash).map(Time).ok_or(NotFound { hash: *hash })
    }

    /// Create a Time from a commit hash, panicking if not found.
    /// Use this for compile-time known commits that must exist.
    pub const fn from_hash_const(hash: &Hash) -> Self {
        match get_index_const(hash) {
            Some(idx) => Time(idx),
            None => panic!("commit not found in history"),
        }
    }

    /// Get the raw time index value
    pub const fn index(&self) -> u16 {
        self.0
    }

    /// Check if this commit time is within a range
    pub fn in_range<R>(&self, range: R) -> bool
    where
        R: std::ops::RangeBounds<Self>,
    {
        range.contains(self)
    }
}

impl PartialOrd for Time {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Time {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse ordering: lower index = earlier = "less than"
        other.0.cmp(&self.0)
    }
}

/// Check if the working directory was dirty at build time
pub fn dirty() -> bool {
    DIRTY.parse().unwrap_or(false)
}

/// Get the commit history as a slice (zero allocation)
fn history() -> &'static [[u8; 32]] {
    // SAFETY: HISTORY_BYTES is guaranteed to be HISTORY_LEN * 32 bytes by build.rs
    // We transmute the byte slice into a slice of [u8; 32] arrays
    unsafe { std::slice::from_raw_parts(HISTORY_BYTES.as_ptr() as *const [u8; 32], HISTORY_LEN) }
}

/// Get the index of a commit in the history (private, for internal use)
fn get_index(hash: &Hash) -> Option<u16> {
    history()
        .iter()
        .position(|h| h == hash)
        .and_then(|idx| u16::try_from(idx).ok())
}

/// Const version of get_index for compile-time usage
const fn get_index_const(hash: &Hash) -> Option<u16> {
    let mut i = 0;
    while i < HISTORY_LEN {
        let offset = i * 32;
        let mut matches = true;
        let mut j = 0;
        while j < 32 {
            if HISTORY_BYTES[offset + j] != hash[j] {
                matches = false;
                break;
            }
            j += 1;
        }
        if matches {
            return Some(i as u16);
        }
        i += 1;
    }
    None
}

const fn parse_commit_hash(hex: &str) -> Hash {
    let mut bytes = [0u8; 32];
    let hex_bytes = hex.as_bytes();
    let mut i = 0;
    while i < 32 && i * 2 + 1 < hex_bytes.len() {
        let high = hex_digit_to_u8(hex_bytes[i * 2]);
        let low = hex_digit_to_u8(hex_bytes[i * 2 + 1]);
        bytes[i] = (high << 4) | low;
        i += 1;
    }
    bytes
}

const fn hex_digit_to_u8(byte: u8) -> u8 {
    match byte {
        b'0'..=b'9' => byte - b'0',
        b'a'..=b'f' => byte - b'a' + 10,
        b'A'..=b'F' => byte - b'A' + 10,
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_commit_hash_size() {
        assert_eq!(COMMIT.len(), 32);
    }

    #[test]
    fn test_commit_time_ordering() {
        let earlier = Time(100);
        let later = Time(50);
        assert!(later > earlier);
        assert!(earlier < later);
    }

    #[test]
    fn test_dirty_flag() {
        // Just ensure it doesn't panic
        let _ = dirty();
    }

    #[test]
    fn test_history_access() {
        let hist = history();
        println!("History length: {}", hist.len());
        assert_eq!(hist.len(), HISTORY_LEN);
    }

    #[test]
    fn test_commit_time_from_hash() {
        // Current commit should be at index 0
        let time = Time::from_hash(&COMMIT).unwrap();
        assert_eq!(time.index(), 0);
    }

    #[test]
    fn test_commit_time_ranges() {
        // Remember: lower index = later in time (reversed ordering)
        // But ranges still use Ord comparison (late < mid < early)
        let late = Time(10); // newer commit (compares as greater)
        let mid = Time(50); // middle commit
        let early = Time(100); // older commit (compares as less)

        // For ranges, start must be <= end in terms of Ord
        // early < mid < late (in Ord terms, because of reversed ordering)
        assert!(mid.in_range(early..=late));
        assert!(!early.in_range(mid..late));
        assert!(!late.in_range(early..mid));
        assert!(early.in_range(early..=mid));
    }

    #[test]
    fn test_const_from_hash() {
        // Test that const version works
        let const_time = Time::from_hash_const(&COMMIT);
        let runtime_time = Time::from_hash(&COMMIT).unwrap();
        assert_eq!(const_time.index(), runtime_time.index());
    }

    #[test]
    fn test_match_with_const_ranges() {
        // Test that const indices work in match patterns
        const V2_IDX: u16 = 50;
        const V3_IDX: u16 = 10;

        let test_cases = vec![
            (Time(5), "v3+"),
            (Time(10), "v3+"),
            (Time(30), "v2"),
            (Time(50), "v1"),
            (Time(75), "v1"),
            (Time(100), "v1"),
            (Time(150), "v1"),
        ];

        for (time, expected) in test_cases {
            let result = match time.index() {
                ..=V3_IDX => "v3+",
                ..V2_IDX => "v2",
                _ => "v1",
            };
            assert_eq!(result, expected, "failed for index {}", time.index());
        }
    }
}
