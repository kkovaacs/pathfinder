use std::sync::{Arc, Mutex};

use cached::{Cached, TimedSizedCache};
use pathfinder_common::TransactionHash;

#[derive(Clone, Debug)]
pub struct SubmittedTransactionTracker(Arc<Mutex<TimedSizedCache<TransactionHash, ()>>>);

impl SubmittedTransactionTracker {
    pub fn new(limit_size: usize, limit_sec: u64) -> Self {
        Self(Arc::new(Mutex::new(
            TimedSizedCache::with_size_and_lifespan(limit_size, limit_sec),
        )))
    }

    pub fn contains_key(&self, hash: &TransactionHash) -> bool {
        let mut cache = self.0.lock().unwrap();
        let res = cache.cache_get(hash);
        res.is_some()
    }

    pub fn insert_key(&self, hash: TransactionHash) {
        let mut cache = self.0.lock().unwrap();
        cache.flush();
        let res = cache.cache_set(hash, ());
        if res.is_some() {
            tracing::warn!("repeated tx hash in mempool: {}", hash);
        }
    }

    pub fn flush(&self) {
        let mut cache = self.0.lock().unwrap();
        cache.flush();
    }
}

#[cfg(test)]
mod tests {
    use pathfinder_common::TransactionHash;
    use pathfinder_crypto::Felt;
    use tokio::time::Duration;

    use super::SubmittedTransactionTracker;

    #[test]
    fn test_full() {
        let tt = SubmittedTransactionTracker::new(2, 10);
        let mut hash = Default::default();
        assert!(!tt.contains_key(&hash));
        for i in 1..=10 {
            hash = TransactionHash(Felt::from_u64(i));
            tt.insert_key(hash);
        }

        assert!(tt.contains_key(&hash));
        hash = TransactionHash(Felt::from_u64(1));
        assert!(!tt.contains_key(&hash));
    }

    #[tokio::test]
    async fn test_flush() {
        let tt = SubmittedTransactionTracker::new(2, 1);
        let hash = TransactionHash(Felt::from_u64(42));
        tt.insert_key(hash);
        assert!(tt.contains_key(&hash));
        tokio::time::sleep(Duration::from_millis(3000)).await;
        tt.flush();
        assert!(!tt.contains_key(&hash));
    }
}
