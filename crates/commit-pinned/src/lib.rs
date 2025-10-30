use commit_info::{CommitHash, CommitNotFound, CommitTime, commit, dirty};
use serde::{Deserialize, Serialize};

/// A wrapper that pins content to a specific git commit.
/// Tracks the commit hash, dirty status, and allows temporal ordering via CommitTime.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CommitPinned<T> {
    /// The git commit hash (SHA-256)
    commit: CommitHash,
    /// Whether the working directory was dirty at build time
    dirty: bool,
    /// The wrapped content
    content: T,
}

impl<T> CommitPinned<T> {
    /// Create a new `CommitPinned` instance with the current build-time git commit info.
    pub fn new(content: T) -> Self {
        Self {
            commit: commit(),
            dirty: dirty(),
            content,
        }
    }

    /// Get the commit hash.
    pub fn commit(&self) -> &CommitHash {
        &self.commit
    }

    /// Get the commit hash as a hex string.
    pub fn commit_hex(&self) -> String {
        hex_encode(&self.commit)
    }

    /// Get the commit time for temporal ordering.
    pub fn commit_time(&self) -> Result<CommitTime, CommitNotFound> {
        CommitTime::from_hash(&self.commit)
    }

    /// Check if the working directory was dirty at build time.
    pub fn dirty(&self) -> bool {
        self.dirty
    }

    /// Get a reference to the content.
    pub fn content(&self) -> &T {
        &self.content
    }

    /// Get a mutable reference to the content.
    pub fn content_mut(&mut self) -> &mut T {
        &mut self.content
    }

    /// Decompose into individual parts.
    pub fn into_parts(self) -> (CommitHash, bool, T) {
        (self.commit, self.dirty, self.content)
    }

    /// Extract the content, discarding the commit information.
    pub fn into_content(self) -> T {
        self.content
    }

    /// Map the content using a function, preserving commit info.
    pub fn map<U, F>(self, f: F) -> CommitPinned<U>
    where
        F: FnOnce(T) -> U,
    {
        CommitPinned {
            commit: self.commit,
            dirty: self.dirty,
            content: f(self.content),
        }
    }
}

/// Compare two CommitPinned values by their commit time (if available).
impl<T> PartialOrd for CommitPinned<T>
where
    T: PartialEq,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let self_time = CommitTime::from_hash(&self.commit).ok()?;
        let other_time = CommitTime::from_hash(&other.commit).ok()?;
        Some(self_time.cmp(&other_time))
    }
}

fn hex_encode(bytes: &[u8; 32]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_and_accessors() {
        let pinned = CommitPinned::new(42);
        assert_eq!(*pinned.content(), 42);
        assert_eq!(pinned.commit().len(), 32);
        assert_eq!(pinned.commit_hex().len(), 64);
    }

    #[test]
    fn test_into_content() {
        let pinned = CommitPinned::new("hello");
        assert_eq!(pinned.into_content(), "hello");
    }

    #[test]
    fn test_into_parts() {
        let pinned = CommitPinned::new(100);
        let (commit, dirty, content) = pinned.into_parts();
        assert_eq!(commit.len(), 32);
        assert_eq!(content, 100);
        println!("commit: {}, dirty: {}", hex_encode(&commit), dirty);
    }

    #[test]
    fn test_map() {
        let pinned = CommitPinned::new(5);
        let mapped = pinned.map(|x| x * 2);
        assert_eq!(*mapped.content(), 10);
    }

    #[test]
    fn test_serde() {
        let pinned = CommitPinned::new("test data".to_string());
        let json = serde_json::to_string(&pinned).unwrap();
        let deserialized: CommitPinned<String> = serde_json::from_str(&json).unwrap();
        assert_eq!(pinned, deserialized);
    }

    #[test]
    fn test_commit_time() {
        let pinned = CommitPinned::new(42);
        let time = pinned.commit_time().unwrap();
        println!("CommitTime: {:?}", time);
    }
}
