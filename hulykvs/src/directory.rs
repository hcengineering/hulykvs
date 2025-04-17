use std::{
    fs,
    io::Result,
    path::{Path, PathBuf},
};

use tokio::{fs::File, io};

use super::{Entry, Key, KeyValueStore, Value};

#[derive(Clone)]
pub struct DirectoryKeyValueStore {
    base: PathBuf,
}

impl DirectoryKeyValueStore {
    pub fn new<P: AsRef<Path>>(base: P) -> Result<Self> {
        let base = base.as_ref().to_path_buf();

        fs::create_dir_all(&base)?;

        Ok(DirectoryKeyValueStore { base })
    }

    pub fn join(&self, p: impl AsRef<Path>) -> Result<Self> {
        Self::new(self.base.join(p.as_ref()))
    }

    fn file_path<K: AsRef<[u8]>>(&self, key: K) -> PathBuf {
        self.base.join(base64_url::encode(key.as_ref()))
    }
}

impl KeyValueStore for DirectoryKeyValueStore {
    async fn insert<K: Into<Key>, V: Into<Value>>(&self, key: K, value: V) -> Result<()> {
        let path = self.file_path(key.into());

        let mut file = File::create(path).await?;
        io::copy(&mut value.into().bytes.as_slice(), &mut file).await?;

        Ok(())
    }

    async fn remove<K: Into<Key>>(&self, key: K) -> Result<bool> {
        let path = self.file_path(key.into());
        match std::fs::remove_file(&path) {
            Ok(()) => Ok(true),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
            Err(e) => Err(e),
        }
    }

    async fn get<K: AsRef<[u8]>>(&self, key: K) -> Result<Option<Entry>> {
        let path = self.file_path(key.as_ref());
        match File::open(&path).await {
            Ok(file) => {
                let value = Value::from_reader(file).await?;
                Ok(Some(Entry {
                    key: Key::new(key.as_ref()),
                    value,
                    md5: None,
                }))
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e),
        }
    }

    async fn exists<K: AsRef<[u8]>>(&self, key: K) -> Result<bool> {
        Ok(self.file_path(key).exists())
    }

    async fn list<K: AsRef<[u8]>>(&self, prefix: K) -> Result<impl Iterator<Item = Key>> {
        let mut result = Vec::new();

        for path in fs::read_dir(&self.base)? {
            let path = path?.path();

            if path.is_file() {
                let file_name = path.file_name().unwrap().to_str().unwrap();
                let key = base64_url::decode(file_name).unwrap();

                if key.starts_with(prefix.as_ref()) {
                    result.push(key.into());
                }
            }
        }

        Ok(result.into_iter())
    }
}
