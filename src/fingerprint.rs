use lazycell::LazyCell;
use std::cmp::PartialEq;
use std::hash::{Hash, Hasher};
use std::io;
use std::path::Path;

/// A lazily-constructed file fingerprint.
///
/// Each fingerprint consists of the length of a file and its hash. Discovery of the hash may be
/// deferred until a hash comparison is actually required. For this reason, each such fingerprint
/// must also store a reference to the path to which it refers.
pub struct LazyFingerprint<'path> {
    path: &'path Path,
    length: u64,
    hash: LazyCell<Vec<u8>>,
}

impl<'a> LazyFingerprint<'a> {
    /// Attempts to create a `LazyFingerprint` from a path.
    ///
    /// This method will fail if the provided path does not reference a file.
    pub fn try_from_path(path: &'a Path) -> io::Result<Self> {
        let meta = path.metadata()?;
        if meta.file_type().is_file() {
            Ok(LazyFingerprint {
                path,
                length: meta.len(),
                hash: LazyCell::new(),
            })
        } else {
            Err(io::Error::new(
                io::ErrorKind::Other,
                "Fingerprint must be derived from a file",
            ))
        }
    }

    pub fn path(&self) -> &Path {
        self.path
    }

    fn sha2_hash(&self) -> &[u8] {
        // I created this method, you know. This is one of my only contributions to the Rust
        // ecosystem outside my own crates!
        self.hash.borrow_with(|| self.derive_hash())
    }

    fn derive_hash(&self) -> Vec<u8> {
        use sha2::{Digest, Sha256};
        use std::cmp;
        use std::fs::File;
        use std::io::{Read, Seek, SeekFrom};

        const MAX_SIZE: u64 = 0x0080_0000; // 8 megabytes

        // In theory, we shouldn't have any problem opening this file because we've already
        // checked to see that a file exists at this location. Anyway... Cross your fingers.
        let mut file = File::open(self.path).unwrap();
        let mut buf = vec![0u8; MAX_SIZE as usize];
        let mut hasher = Sha256::default();

        // First, apply the first eight megabytes of the file.
        let len = file.read(&mut buf).unwrap();
        hasher.input(&buf[..len]);

        // If there is any data remaining, read the tail of the file and process that as well.
        if self.length > MAX_SIZE {
            let remaining = cmp::min(MAX_SIZE, self.length - MAX_SIZE);

            // Rust's documentation does not make clear the operation of this method. I'll need
            // to verify its behavior at some point.
            file.seek(SeekFrom::End(-(remaining as i64))).unwrap();

            let len = file.read(&mut buf).unwrap();
            hasher.input(&buf[..len]);
        }

        hasher.result().into_iter().collect()
    }
}

impl Eq for LazyFingerprint<'_> {}

impl PartialEq for LazyFingerprint<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.length == other.length && self.sha2_hash() == other.sha2_hash()
    }
}

impl Hash for LazyFingerprint<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.length.hash(state);

        // I think I want to try hashing based solely on file length, then throw in the sha2
        // hash only in the case of collisions. This may increase the likelihood that hashing
        // can actually be deferred or skipped.
        // self.sha2_hash().hash(state);
    }
}
