use std::sync::Arc;

use dashmap::DashMap;
use neptune_privacy::api::export::SpendingKey;

pub(super) struct KeyCache {
    symmetric_keys: DashMap<u64, Arc<SpendingKey>>,
    generation_spending_keys: DashMap<u64, Arc<SpendingKey>>,
}

impl KeyCache {
    pub fn new() -> Self {
        Self {
            symmetric_keys: DashMap::new(),
            generation_spending_keys: DashMap::new(),
        }
    }
    pub fn get_symmetric_key(&self, index: u64) -> Option<Arc<SpendingKey>> {
        self.symmetric_keys.get(&index).map(|d| d.value().clone())
    }
    pub fn get_generation_spending_key(&self, index: u64) -> Option<Arc<SpendingKey>> {
        self.generation_spending_keys
            .get(&index)
            .map(|d| d.value().clone())
    }

    pub fn add_symmetric_key(&self, index: u64, key: Arc<SpendingKey>) {
        self.symmetric_keys.insert(index, key);
    }

    pub fn add_generation_spending_key(&self, index: u64, key: Arc<SpendingKey>) {
        self.generation_spending_keys.insert(index, key);
    }
}
