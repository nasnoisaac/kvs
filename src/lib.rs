use std::collections::HashMap;

/// The KvStore is a simple key-value store.
/// # Example:
/// ```rust
/// use kvs::KvStore;
///
/// let mut store = KvStore::new();
///
/// store.set("key".to_string(), "test".to_string());
/// store.get("key".to_string());
/// store.remove("key".to_string());
/// ```
#[derive(Default)]
pub struct KvStore {
    store: HashMap<String, String>,
}

impl KvStore {
    pub fn new() -> Self {
        KvStore {
            store: HashMap::new(),
        }
    }

    pub fn set(&mut self, key: String, value: String) {
        self.store.insert(key, value);
    }

    pub fn get(&self, key: String) -> Option<String> {
        self.store.get(&key).cloned()
    }

    pub fn remove(&mut self, key: String) {
        self.store.remove(&key);
    }
}
