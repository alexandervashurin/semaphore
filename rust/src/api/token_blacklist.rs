//! JWT Token Blacklist — in-memory implementation (v5.0)
//!
//! Stores revoked JTIs with their expiry timestamp.
//! Background task prunes expired entries every 5 minutes.
//!
//! Usage:
//! - On logout: `blacklist.revoke(jti, exp)`
//! - On verify: `if blacklist.is_revoked(jti) { return Err(Unauthorized) }`

use dashmap::DashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time;

/// In-memory JWT blacklist backed by DashMap
#[derive(Clone)]
pub struct TokenBlacklist {
    /// jti → expiry instant (when to auto-remove)
    inner: Arc<DashMap<String, Instant>>,
}

impl Default for TokenBlacklist {
    fn default() -> Self {
        Self::new()
    }
}

impl TokenBlacklist {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(DashMap::new()),
        }
    }

    /// Revoke a token by its JTI. `exp` is Unix timestamp (seconds) from the JWT claim.
    pub fn revoke(&self, jti: &str, exp: usize) {
        let now_unix = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as usize;

        if exp <= now_unix {
            // Already expired — no need to store
            return;
        }

        let ttl = Duration::from_secs((exp - now_unix) as u64);
        let expiry = Instant::now() + ttl;
        self.inner.insert(jti.to_string(), expiry);
    }

    /// Returns true if the JTI has been explicitly revoked and the token hasn't expired yet.
    pub fn is_revoked(&self, jti: &str) -> bool {
        match self.inner.get(jti) {
            Some(expiry) => Instant::now() < *expiry,
            None => false,
        }
    }

    /// Remove expired entries. Called periodically by the pruner task.
    pub fn prune(&self) {
        let now = Instant::now();
        self.inner.retain(|_, expiry| *expiry > now);
    }

    /// Returns count of currently blacklisted (non-expired) tokens.
    pub fn len(&self) -> usize {
        let now = Instant::now();
        self.inner.iter().filter(|e| *e.value() > now).count()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Spawns a background task that prunes the blacklist every `interval`.
pub fn spawn_pruner(blacklist: TokenBlacklist, interval: Duration) {
    tokio::spawn(async move {
        let mut ticker = time::interval(interval);
        loop {
            ticker.tick().await;
            let before = blacklist.inner.len();
            blacklist.prune();
            let after = blacklist.inner.len();
            if before != after {
                tracing::debug!("JWT blacklist pruned: {} → {} entries", before, after);
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn revoke_marks_jti_until_exp() {
        let b = TokenBlacklist::new();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize;
        let jti = "test-jti-1";
        b.revoke(jti, now + 3600);
        assert!(b.is_revoked(jti));
        assert!(!b.is_revoked("other-jti"));
    }

    #[test]
    fn expired_token_not_stored() {
        let b = TokenBlacklist::new();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize;
        b.revoke("old", now.saturating_sub(10));
        assert!(!b.is_revoked("old"));
    }
}
