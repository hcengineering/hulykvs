use std::io::Result;

use dashmap::DashMap;

use super::{Entry, Key, KeyValueStore, Value};

#[derive(Default)]
pub struct MemoryKeyValueStore {
    store: DashMap<Key, Value>,
}

impl KeyValueStore for MemoryKeyValueStore {
    async fn insert<K: Into<Key>, V: Into<Value>>(&self, key: K, value: V) -> Result<()> {
        self.store.insert(key.into(), value.into());

        Ok(())
    }

    async fn remove<K: Into<Key>>(&self, key: K) -> Result<bool> {
        Ok(self.store.remove(&key.into()).is_some())
    }

    async fn get<K: AsRef<[u8]>>(&self, key: K) -> Result<Option<Entry>> {
        Ok(self.store.get(&Key::new(key.as_ref())).map(|v| Entry {
            key: v.key().clone(),
            value: v.value().clone(),
            md5: None,
        }))
    }

    async fn exists<K: AsRef<[u8]>>(&self, key: K) -> Result<bool> {
        Ok(self.store.contains_key(&Key::new(key.as_ref())))
    }

    async fn list<K: AsRef<[u8]>>(&self, prefix: K) -> Result<impl Iterator<Item = Key>> {
        Ok(self
            .store
            .iter()
            .filter(move |entry| entry.key().bytes.starts_with(prefix.as_ref()))
            .map(|entry| entry.key().to_owned()))
    }
}

/*
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_put_and_get() -> Result<()> {
        let store = MemoryKeyValueStore::new();
        let key = b"test_key";
        let value = b"test_value";

        store.put(key, value.as_slice()).await?;

        let entry = store.get(&key).await?;
        assert!(entry.is_some());

        let bytes = entry.unwrap().to_bytes().await?;
        assert_eq!(bytes, value);

        Ok(())
    }

    #[tokio::test]
    async fn test_delete() -> Result<()> {
        let store = MemoryKeyValueStore::new();
        let key = b"delete_key";
        let value = b"delete_value";

        store.put(&key, value.as_slice()).await?;
        assert!(store.exists(&key).await?);

        let deleted = store.delete(&key).await?;
        assert!(deleted);
        assert!(!store.exists(&key).await?);

        let deleted = store.delete(&key).await?;
        assert!(!deleted);

        Ok(())
    }

    #[tokio::test]
    async fn test_exists() -> Result<()> {
        let store = MemoryKeyValueStore::new();
        let key = b"exists_key";
        let non_existent_key = b"non_existent";

        assert!(!store.exists(&key).await?);

        store.put(&key, b"some_value".as_slice()).await?;
        assert!(store.exists(&key).await?);

        assert!(!store.exists(&non_existent_key).await?);

        Ok(())
    }

    #[tokio::test]
    async fn test_list() -> Result<()> {
        let store = MemoryKeyValueStore::new();

        let prefixed_keys = vec![
            b"prefix_one".to_vec(),
            b"prefix_two".to_vec(),
            b"prefix_three".to_vec(),
        ];

        let other_keys = vec![b"other_one".to_vec(), b"other_two".to_vec()];

        for key in prefixed_keys.iter().chain(other_keys.iter()) {
            store.put(key, b"value".as_slice()).await?;
        }

        let all_keys: Vec<_> = store.list().await?.collect();
        assert_eq!(all_keys.len(), 5);

        let prefix = b"prefix";
        let filtered_keys: Vec<_> = store.list_prefixed(prefix).await?.collect();
        assert_eq!(filtered_keys.len(), 3);

        for key in filtered_keys {
            assert!(String::from_utf8_lossy(&key).starts_with("prefix"));
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_empty_key_and_value() -> Result<()> {
        let store = MemoryKeyValueStore::new();
        let empty_key = b"";
        let empty_value = b"";

        store.put(b"key", empty_value.as_slice()).await?;
        let entry = store.get(b"key").await?;
        assert!(entry.is_some());

        let bytes = entry.unwrap().to_bytes().await?;
        assert_eq!(bytes, empty_value);

        store.put(empty_key, b"value".as_slice()).await?;
        let entry = store.get(&empty_key).await?;
        assert!(entry.is_some());

        Ok(())
    }

    #[tokio::test]
    async fn test_overwrite_value() -> Result<()> {
        let store = MemoryKeyValueStore::new();
        let key = b"overwrite_key";

        store.put(key, b"first_value".as_slice()).await?;

        store.put(key, b"second_value".as_slice()).await?;

        let entry = store.get(key).await?.unwrap();
        let bytes = entry.to_bytes().await?;

        assert_eq!(bytes, b"second_value");

        Ok(())
    }

    #[tokio::test]
    async fn test_non_existent_key() -> Result<()> {
        let store = MemoryKeyValueStore::new();
        let key = b"non_existent";

        let entry = store.get(key).await?;
        assert!(entry.is_none());

        Ok(())
    }
}
*/
