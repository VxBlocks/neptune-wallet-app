use std::{collections::BTreeMap, sync::Arc};

use serde::{de::DeserializeOwned, Serialize};
use tokio::sync::RwLock;

pub mod command;
pub mod persist;

#[derive(Debug, Clone)]
pub struct Memstore {
    sessions: Arc<RwLock<BTreeMap<Vec<u8>, Vec<u8>>>>,
}

impl Memstore {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(BTreeMap::new())),
        }
    }

    pub async fn get<K: Serialize, V: DeserializeOwned>(&self, key: &K) -> Option<V> {
        let key = serde_json::to_vec(key).unwrap();
        let sessions = self.sessions.read().await;
        let value = sessions.get(&key).cloned();
        match value {
            Some(value) => {
                let value: V = serde_json::from_slice(&value).unwrap();
                Some(value)
            }
            None => None,
        }
    }

    pub async fn set<K: Serialize, V: Serialize>(&self, key: &K, value: &V) {
        let key = serde_json::to_vec(key).unwrap();
        let value = serde_json::to_vec(value).unwrap();
        let mut sessions = self.sessions.write().await;
        sessions.insert(key.to_vec(), value.to_vec());
    }

    pub async fn del<K: Serialize, V: DeserializeOwned>(&self, key: &K) -> Option<V> {
        let key = serde_json::to_vec(key).unwrap();
        let mut sessions = self.sessions.write().await;
        let value = sessions.remove(&key);
        match value {
            Some(value) => {
                let value: V = serde_json::from_slice(&value).unwrap();
                Some(value)
            }
            None => None,
        }
    }
}
