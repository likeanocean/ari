use ring::digest::{Algorithm, Context, Digest};
use std::io::Read;
use std::ops::Deref;

pub enum HashAlgorithm {
    Sha1,
    Sha256,
    Sha384,
    Sha512,
}

impl Into<&'static Algorithm> for HashAlgorithm {
    fn into(self) -> &'static Algorithm {
        match self {
            HashAlgorithm::Sha1 => &ring::digest::SHA1_FOR_LEGACY_USE_ONLY,
            HashAlgorithm::Sha256 => &ring::digest::SHA256,
            HashAlgorithm::Sha384 => &ring::digest::SHA384,
            HashAlgorithm::Sha512 => &ring::digest::SHA512,
        }
    }
}

pub struct IncrementalHash {
    context: Context,
}

impl IncrementalHash {
    pub fn new(algorithm: HashAlgorithm) -> IncrementalHash {
        let algorithm = algorithm.into();
        let context = Context::new(algorithm);

        IncrementalHash { context }
    }

    pub fn update(&mut self, data: &[u8]) {
        self.context.update(data);
    }

    pub fn finish(self) -> Hash {
        let digest = self.context.finish();

        Hash { digest }
    }
}

pub struct Hash {
    digest: Digest,
}

impl AsRef<[u8]> for Hash {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.digest.as_ref()
    }
}

impl Deref for Hash {
    type Target = [u8];

    #[inline]
    fn deref(&self) -> &[u8] {
        self.digest.as_ref()
    }
}

pub fn hash_slice(data: &[u8], algorithm: HashAlgorithm) -> Hash {
    let digest = ring::digest::digest(algorithm.into(), data);

    Hash { digest }
}

// todo: const-generics: make `BUFFER_LENGTH` a generic parameter with default value.
pub fn hash_read(source: &mut impl Read, algorithm: HashAlgorithm) -> Result<Hash, std::io::Error> {
    const BUFFER_LENGTH: usize = 1024 * 1024;

    let mut buffer = vec![0; BUFFER_LENGTH];
    let mut hash = IncrementalHash::new(algorithm);

    loop {
        match source.read(&mut buffer)? {
            0 => return Ok(hash.finish()),
            read => hash.update(&buffer[0..read]),
        }
    }
}
