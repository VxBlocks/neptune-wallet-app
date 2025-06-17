use std::sync::Arc;

use dashmap::DashMap;
use neptune_cash::models::state::wallet::address::BaseSpendingKey;

pub(super) struct KeyCache {
    symmetric_keys: DashMap<u64, Arc<BaseSpendingKey>>,
    generation_spending_keys: DashMap<u64, Arc<BaseSpendingKey>>,
}

impl KeyCache {
    pub fn new() -> Self {
        Self {
            symmetric_keys: DashMap::new(),
            generation_spending_keys: DashMap::new(),
        }
    }
    pub fn get_symmetric_key(&self, index: u64) -> Option<Arc<BaseSpendingKey>> {
        self.symmetric_keys.get(&index).map(|d| d.value().clone())
    }
    pub fn get_generation_spending_key(&self, index: u64) -> Option<Arc<BaseSpendingKey>> {
        self.generation_spending_keys
            .get(&index)
            .map(|d| d.value().clone())
    }

    pub fn add_symmetric_key(&self, index: u64, key: Arc<BaseSpendingKey>) {
        self.symmetric_keys.insert(index, key);
    }

    pub fn add_generation_spending_key(&self, index: u64, key: Arc<BaseSpendingKey>) {
        self.generation_spending_keys.insert(index, key);
    }
}
