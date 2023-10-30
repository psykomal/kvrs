use std::path::PathBuf;
use std::sync::Arc;

use crate::KvsEngine;
use crate::Result;
use crossbeam_skiplist::SkipMap;

#[derive(Clone)]
pub struct InMemEngine {
    store: Arc<SkipMap<String, String>>,
}

impl InMemEngine {
    pub fn open(path: PathBuf) -> Self {
        Self {
            store: Arc::new(SkipMap::new()),
        }
    }
}

impl KvsEngine for InMemEngine {
    fn get(&self, key: String) -> Result<Option<String>> {
        match self.store.get(&key) {
            Some(entry) => Ok(Some(entry.value().clone())),
            None => Ok(None),
        }
    }

    fn set(&self, key: String, value: String) -> Result<()> {
        self.store.insert(key, value);
        Ok(())
    }

    fn remove(&self, key: String) -> Result<()> {
        match self.store.remove(&key) {
            Some(_) => Ok(()),
            None => Err(failure::err_msg("Key not found")),
        }
    }
}
