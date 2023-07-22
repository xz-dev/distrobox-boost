use std::collections::HashMap;
use std::hash::Hash;
use std::sync::{Arc, Mutex};

pub struct SynchronizedMap<K: Eq + Hash> {
    map: Mutex<HashMap<K, Arc<Mutex<()>>>>,
}

impl<K: Eq + Hash> SynchronizedMap<K> {
    pub fn new() -> Self {
        Self {
            map: Mutex::new(HashMap::new()),
        }
    }

    pub fn execute<F, R>(&self, key: K, func: F) -> R
    where
        F: FnOnce() -> R,
        K: Clone,
    {
        let mut map = self.map.lock().unwrap();

        let mutex = if let Some(m) = map.get(&key) {
            m.clone()
        } else {
            let m = Arc::new(Mutex::new(()));
            map.insert(key, m.clone());
            m
        };

        let _guard = mutex.lock().unwrap();
        func()
    }
}
