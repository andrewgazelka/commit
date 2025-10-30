use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

/// A 32-byte SHA-256 hash representing a commit
pub type CommitHash = [u8; 32];

/// Error when a commit hash is not found in the history
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommitNotFound {
    pub hash: CommitHash,
}

impl std::fmt::Display for CommitNotFound {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "commit not found in history: {}",
            self.hash.iter().map(|b| format!("{:02x}", b)).collect::<String>()
        )
    }
}

impl std::error::Error for CommitNotFound {}

const COMMIT: &str = env!("GIT_COMMIT");
const DIRTY: &str = env!("GIT_DIRTY");
const HISTORY_LEN: usize = const_str::parse!(env!("GIT_HISTORY_LEN"), usize);

// Include the raw history bytes at compile time
const HISTORY_BYTES: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/history.bin"));

/// A timestamp index for a commit, allowing temporal ordering based on commit history.
/// Lower values indicate earlier commits in the repository history.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CommitTime(u16);

impl CommitTime {
    /// Create a CommitTime from a commit hash by looking up its position in history
    pub fn from_hash(hash: &CommitHash) -> Result<Self, CommitNotFound> {
        get_index(hash)
            .map(CommitTime)
            .ok_or(CommitNotFound { hash: *hash })
    }

    /// Create a CommitTime from a commit hash, panicking if not found.
    /// Use this for compile-time known commits that must exist.
    pub const fn from_hash_const(hash: &CommitHash) -> Self {
        match get_index_const(hash) {
            Some(idx) => CommitTime(idx),
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

impl PartialOrd for CommitTime {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for CommitTime {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse ordering: lower index = earlier = "less than"
        other.0.cmp(&self.0)
    }
}

/// Get the current commit hash at build time
pub fn commit() -> CommitHash {
    parse_commit_hash(COMMIT)
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
fn get_index(hash: &CommitHash) -> Option<u16> {
    history()
        .iter()
        .position(|h| h == hash)
        .and_then(|idx| u16::try_from(idx).ok())
}

/// Const version of get_index for compile-time usage
const fn get_index_const(hash: &CommitHash) -> Option<u16> {
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

fn parse_commit_hash(hex: &str) -> CommitHash {
    let mut bytes = [0u8; 32];
    for (i, chunk) in hex.as_bytes().chunks(2).enumerate() {
        if i >= 32 {
            break;
        }
        if chunk.len() == 2 {
            bytes[i] =
                u8::from_str_radix(std::str::from_utf8(chunk).unwrap_or("00"), 16).unwrap_or(0);
        }
    }
    bytes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_commit_hash_size() {
        let hash = commit();
        assert_eq!(hash.len(), 32);
    }

    #[test]
    fn test_commit_time_ordering() {
        let earlier = CommitTime(100);
        let later = CommitTime(50);
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
        let current = commit();
        // Current commit should be at index 0
        let time = CommitTime::from_hash(&current).unwrap();
        assert_eq!(time.index(), 0);
    }

    #[test]
    fn test_commit_time_ranges() {
        // Remember: lower index = later in time (reversed ordering)
        // But ranges still use Ord comparison (late < mid < early)
        let late = CommitTime(10);     // newer commit (compares as greater)
        let mid = CommitTime(50);      // middle commit
        let early = CommitTime(100);   // older commit (compares as less)

        // For ranges, start must be <= end in terms of Ord
        // early < mid < late (in Ord terms, because of reversed ordering)
        assert!(mid.in_range(early..=late));
        assert!(!early.in_range(mid..late));
        assert!(!late.in_range(early..mid));
        assert!(early.in_range(early..=mid));
    }

    #[test]
    fn test_const_from_hash() {
        let current = commit();

        // Test that const version works
        let const_time = CommitTime::from_hash_const(&current);
        let runtime_time = CommitTime::from_hash(&current).unwrap();
        assert_eq!(const_time.index(), runtime_time.index());
    }

    #[test]
    fn test_match_with_const_ranges() {
        // Test that const indices work in match patterns
        const V2_IDX: u16 = 50;
        const V3_IDX: u16 = 10;

        let test_cases = vec![
            (CommitTime(5), "v3+"),
            (CommitTime(10), "v3+"),
            (CommitTime(30), "v2"),
            (CommitTime(50), "v1"),
            (CommitTime(75), "v1"),
            (CommitTime(100), "v1"),
            (CommitTime(150), "v1"),
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
