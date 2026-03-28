use std::{
    error::Error,
    path::PathBuf,
    sync::{Arc, Mutex},
};
use vibe_core::StorageKind;

use crate::store::{RelayStore, load_relay_store, persist_relay_store};

pub(crate) trait RelayStorage: Send + Sync {
    fn load(&self) -> Result<RelayStore, Box<dyn Error>>;
    fn save(&self, store: &RelayStore) -> Result<(), Box<dyn Error>>;
    fn descriptor(&self) -> String;
}

pub(crate) struct FileRelayStorage {
    path: PathBuf,
}

impl FileRelayStorage {
    pub(crate) fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }
}

impl RelayStorage for FileRelayStorage {
    fn load(&self) -> Result<RelayStore, Box<dyn Error>> {
        load_relay_store(&self.path)
    }

    fn save(&self, store: &RelayStore) -> Result<(), Box<dyn Error>> {
        persist_relay_store(&self.path, store)
    }

    fn descriptor(&self) -> String {
        self.path.to_string_lossy().to_string()
    }
}

#[derive(Default)]
pub(crate) struct MemoryRelayStorage {
    snapshot: Mutex<RelayStore>,
}

impl RelayStorage for MemoryRelayStorage {
    fn load(&self) -> Result<RelayStore, Box<dyn Error>> {
        Ok(self.snapshot.lock().unwrap().clone())
    }

    fn save(&self, store: &RelayStore) -> Result<(), Box<dyn Error>> {
        *self.snapshot.lock().unwrap() = store.clone();
        Ok(())
    }

    fn descriptor(&self) -> String {
        "memory://relay-store".to_string()
    }
}

pub(crate) fn build_relay_storage(kind: StorageKind, state_file: PathBuf) -> Arc<dyn RelayStorage> {
    match kind {
        StorageKind::Memory => Arc::new(MemoryRelayStorage::default()),
        StorageKind::File | StorageKind::External => Arc::new(FileRelayStorage::new(state_file)),
    }
}
