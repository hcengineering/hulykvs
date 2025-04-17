use std::io::Result;

use tokio::io::{AsyncRead, AsyncReadExt};

pub mod directory;
pub mod memory;

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct Slice {
    pub bytes: Vec<u8>,
}

impl Slice {
    pub fn new(bytes: &[u8]) -> Self {
        Slice {
            bytes: bytes.to_vec(),
        }
    }

    pub async fn from_reader<R: AsyncRead + Unpin>(mut reader: R) -> Result<Self> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        Ok(Self { bytes })
    }
}

impl From<Vec<u8>> for Slice {
    fn from(bytes: Vec<u8>) -> Self {
        Self { bytes }
    }
}

impl From<Slice> for Vec<u8> {
    fn from(value: Slice) -> Self {
        value.bytes
    }
}

impl AsRef<[u8]> for Slice {
    fn as_ref(&self) -> &[u8] {
        &self.bytes
    }
}

pub type Key = Slice;
pub type Value = Slice;

pub struct Entry {
    pub key: Key,
    pub value: Value,
    pub md5: Option<[u8; 16]>,
}

impl Entry {
    pub fn from_json<T>(self) -> serde_json::Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        serde_json::from_slice(&self.value.bytes)
    }
}

pub trait KeyValueStore {
    fn insert<K: Into<Key>, V: Into<Value>>(
        &self,
        key: K,
        value: V,
    ) -> impl Future<Output = Result<()>>;

    fn remove<K: Into<Key>>(&self, key: K) -> impl Future<Output = Result<bool>>;

    fn exists<K: AsRef<[u8]>>(&self, key: K) -> impl Future<Output = Result<bool>>;

    fn get<K: AsRef<[u8]>>(&self, key: K) -> impl Future<Output = Result<Option<Entry>>>;

    fn list<K: AsRef<[u8]>>(
        &self,
        prefix: K,
    ) -> impl Future<Output = Result<impl Iterator<Item = Key>>>;
}
