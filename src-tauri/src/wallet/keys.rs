use std::ops::Deref;
use std::range::Range;
use std::sync::atomic::Ordering;
use std::sync::Arc;

use anyhow::Result;
use neptune_privacy::api::export::SpendingKey;
use rayon::prelude::*;

impl super::WalletState {
    pub async fn get_address(&self, index: u64) -> Result<String> {
        let symmetric_key = self.key.nth_generation_spending_key(index);
        let spending_key = SpendingKey::from(symmetric_key);

        spending_key.to_address().to_bech32m(self.network)
    }

    pub fn get_known_spending_keys(&self) -> Vec<SpendingKey> {
        let spending_keys = self.get_future_generation_spending_keys(Range {
            start: 0,
            end: self.num_generation_spending_keys() + 1,
        });
        let spending_keys = spending_keys.iter().map(|v| v.1.deref().clone());

        let symmetric_keys = self.get_future_symmetric_keys(Range {
            start: 0,
            end: self.num_symmetric_keys() + 1,
        });
        let symmetric_keys = symmetric_keys.iter().map(|v| v.1.deref().clone());

        // let raw_hash_keys = self.get_known_raw_hash_keys();

        spending_keys
            .chain(symmetric_keys)
            // .chain(raw_hash_keys)
            .collect()
    }

    pub fn num_symmetric_keys(&self) -> u64 {
        self.num_symmetric_keys.load(Ordering::Relaxed)
    }

    pub fn num_generation_spending_keys(&self) -> u64 {
        self.num_generation_spending_keys.load(Ordering::Relaxed)
    }

    pub fn num_future_keys(&self) -> u64 {
        self.num_future_keys.load(Ordering::Relaxed)
    }

    pub fn get_future_symmetric_keys(&self, range: Range<u64>) -> Vec<(u64, Arc<SpendingKey>)> {
        let key = &self.key;
        (range.start..range.end)
            .into_par_iter()
            .map(|i| {
                if let Some(key) = self.key_cache.get_symmetric_key(i) {
                    return (i, key);
                }
                let new_key = Arc::new(SpendingKey::from(key.nth_symmetric_key(i)));
                self.key_cache.add_symmetric_key(i, new_key.clone());
                (i, new_key)
            })
            .collect()
    }

    pub fn get_future_generation_spending_keys(
        &self,
        range: Range<u64>,
    ) -> Vec<(u64, Arc<SpendingKey>)> {
        let key = &self.key;
        (range.start..range.end)
            .into_par_iter()
            .map(|i| {
                if let Some(key) = self.key_cache.get_generation_spending_key(i) {
                    return (i, key);
                }
                let new_key = Arc::new(SpendingKey::from(key.nth_generation_spending_key(i)));
                self.key_cache
                    .add_generation_spending_key(i, new_key.clone());
                (i, new_key)
            })
            .collect()
    }
}
