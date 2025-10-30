use commit_info::{Hash, NotFound, Time, commit, dirty};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Placeholder type that ignores content during deserialization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Ignored;

impl Serialize for Ignored {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_unit()
    }
}

impl<'de> Deserialize<'de> for Ignored {
    fn deserialize<D>(_deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Ignore any content
        Ok(Ignored)
    }
}

/// A wrapper that pins content to a specific git commit.
/// Tracks the commit hash, dirty status, and allows temporal ordering via Time.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Pinned<T = Ignored> {
    /// The git commit hash (SHA-256)
    commit: Hash,
    /// Whether the working directory was dirty at build time
    dirty: bool,
    /// The wrapped content
    content: T,
}

impl<T> Pinned<T> {
    /// Create a new `Pinned` instance with the current build-time git commit info.
    pub fn new(content: T) -> Self {
        Self {
            commit: commit(),
            dirty: dirty(),
            content,
        }
    }

    /// Get the commit hash.
    pub fn commit(&self) -> &Hash {
        &self.commit
    }

    /// Get the commit hash as a hex string.
    pub fn commit_hex(&self) -> String {
        hex_encode(&self.commit)
    }

    /// Get the commit time for temporal ordering.
    pub fn commit_time(&self) -> Result<Time, NotFound> {
        Time::from_hash(&self.commit)
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
    pub fn into_parts(self) -> (Hash, bool, T) {
        (self.commit, self.dirty, self.content)
    }

    /// Extract the content, discarding the commit information.
    pub fn into_content(self) -> T {
        self.content
    }

    /// Map the content using a function, preserving commit info.
    pub fn map<U, F>(self, f: F) -> Pinned<U>
    where
        F: FnOnce(T) -> U,
    {
        Pinned {
            commit: self.commit,
            dirty: self.dirty,
            content: f(self.content),
        }
    }
}

/// Compare two Pinned values by their commit time (if available).
impl<T> PartialOrd for Pinned<T>
where
    T: PartialEq,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let self_time = Time::from_hash(&self.commit).ok()?;
        let other_time = Time::from_hash(&other.commit).ok()?;
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
        let pinned = Pinned::new(42);
        assert_eq!(*pinned.content(), 42);
        assert_eq!(pinned.commit().len(), 32);
        assert_eq!(pinned.commit_hex().len(), 64);
    }

    #[test]
    fn test_into_content() {
        let pinned = Pinned::new("hello");
        assert_eq!(pinned.into_content(), "hello");
    }

    #[test]
    fn test_into_parts() {
        let pinned = Pinned::new(100);
        let (commit, dirty, content) = pinned.into_parts();
        assert_eq!(commit.len(), 32);
        assert_eq!(content, 100);
        println!("commit: {}, dirty: {}", hex_encode(&commit), dirty);
    }

    #[test]
    fn test_map() {
        let pinned = Pinned::new(5);
        let mapped = pinned.map(|x| x * 2);
        assert_eq!(*mapped.content(), 10);
    }

    #[test]
    fn test_serde() {
        let pinned = Pinned::new("test data".to_string());
        let json = serde_json::to_string(&pinned).unwrap();
        let deserialized: Pinned<String> = serde_json::from_str(&json).unwrap();
        assert_eq!(pinned, deserialized);
    }

    #[test]
    fn test_commit_time() {
        let pinned = Pinned::new(42);
        let time = pinned.commit_time().unwrap();
        println!("Time: {:?}", time);
    }
}
