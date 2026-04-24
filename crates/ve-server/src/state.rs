//! Application State
//!
//! Shared state for the server including database, WebSocket hub, and configuration.

use std::collections::HashMap;
use std::hash::Hash;
use std::net::IpAddr;
use std::num::NonZeroU32;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use governor::clock::DefaultClock;
use governor::state::keyed::DefaultKeyedStateStore;
use governor::{Quota, RateLimiter};
use uuid::Uuid;
use ve_shared::jwt::JwtManager;

use crate::config::Config;
use crate::db::DbPool;
use crate::hub::Hub;

const REGISTER_DEVICE_THROTTLE_LIMIT: u32 = 5;
const DAEMON_HELLO_THROTTLE_LIMIT: u32 = 5;
const PAIR_DEVICE_THROTTLE_LIMIT: u32 = 5;
const PAIR_CODE_THROTTLE_LIMIT: u32 = 5;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct DevicePairKey {
    pub device_id: Uuid,
    pub pair_code: String,
}

type KeyedLimiter<K> = RateLimiter<K, DefaultKeyedStateStore<K>, DefaultClock>;

pub struct AuthThrottle {
    register_device: KeyedLimiter<IpAddr>,
    daemon_hello: KeyedLimiter<IpAddr>,
    pair_device: KeyedLimiter<Uuid>,
    pair_code: KeyedLimiter<DevicePairKey>,
    missing_pair_devices: Mutex<HashMap<Uuid, Instant>>,
    window: Duration,
}

impl AuthThrottle {
    pub fn new(window: Duration) -> Self {
        Self {
            register_device: build_limiter(REGISTER_DEVICE_THROTTLE_LIMIT, window),
            daemon_hello: build_limiter(DAEMON_HELLO_THROTTLE_LIMIT, window),
            pair_device: build_limiter(PAIR_DEVICE_THROTTLE_LIMIT, window),
            pair_code: build_limiter(PAIR_CODE_THROTTLE_LIMIT, window),
            missing_pair_devices: Mutex::new(HashMap::new()),
            window,
        }
    }

    pub fn allow_register_device(&self, remote_ip: IpAddr) -> bool {
        self.allow_with_cleanup(&self.register_device, remote_ip)
    }

    pub fn allow_daemon_hello(&self, remote_ip: IpAddr) -> bool {
        self.allow_with_cleanup(&self.daemon_hello, remote_ip)
    }

    pub fn allow_pair_device(&self, device_id: Uuid) -> bool {
        self.allow_with_cleanup(&self.pair_device, device_id)
    }

    pub fn allow_pair_code(&self, device_id: Uuid, pair_code: String) -> bool {
        self.allow_with_cleanup(
            &self.pair_code,
            DevicePairKey {
                device_id,
                pair_code,
            },
        )
    }

    pub fn is_known_missing_pair_device(&self, device_id: Uuid) -> bool {
        let mut missing_pair_devices = self.missing_pair_devices.lock().unwrap();
        missing_pair_devices.retain(|_, seen_at| seen_at.elapsed() < self.window);
        missing_pair_devices.contains_key(&device_id)
    }

    pub fn remember_missing_pair_device(&self, device_id: Uuid) {
        let mut missing_pair_devices = self.missing_pair_devices.lock().unwrap();
        missing_pair_devices.retain(|_, seen_at| seen_at.elapsed() < self.window);
        missing_pair_devices.insert(device_id, Instant::now());
    }

    pub fn clear_missing_pair_device(&self, device_id: Uuid) {
        let mut missing_pair_devices = self.missing_pair_devices.lock().unwrap();
        missing_pair_devices.remove(&device_id);
    }

    fn allow_with_cleanup<K>(&self, limiter: &KeyedLimiter<K>, key: K) -> bool
    where
        K: Clone + Eq + Hash,
    {
        limiter.retain_recent();
        limiter.check_key(&key).is_ok()
    }
}

impl Default for AuthThrottle {
    fn default() -> Self {
        Self::new(Duration::from_secs(300))
    }
}

fn build_limiter<K>(limit: u32, window: Duration) -> KeyedLimiter<K>
where
    K: Clone + Eq + Hash,
{
    let quota = Quota::with_period(window)
        .expect("auth throttle period must be non-zero")
        .allow_burst(NonZeroU32::new(limit).expect("auth throttle limit must be non-zero"));

    RateLimiter::keyed(quota)
}

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    /// Database connection pool
    pub db: DbPool,

    /// WebSocket hub for real-time communication
    pub hub: Arc<Hub>,

    /// Server configuration
    pub config: Arc<Config>,

    /// Shared JWT manager reused by HTTP handlers and WS endpoints
    pub jwt_manager: Arc<JwtManager>,

    /// In-memory authentication throttles
    pub auth_throttle: Arc<AuthThrottle>,
}

impl AppState {
    /// Create a new application state
    pub fn new(db: DbPool, hub: Hub, config: Config, jwt_manager: Arc<JwtManager>) -> Self {
        Self {
            db,
            hub: Arc::new(hub),
            config: Arc::new(config.clone()),
            jwt_manager,
            auth_throttle: Arc::new(AuthThrottle::new(Duration::from_secs(
                config.pair_code_ttl_secs,
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};
    use std::thread;

    #[test]
    fn register_device_limits_concurrent_requests_for_same_ip() {
        let throttle = Arc::new(AuthThrottle::new(Duration::from_secs(300)));
        let barrier = Arc::new(std::sync::Barrier::new(9));
        let mut workers = Vec::new();

        for _ in 0..8 {
            let throttle = throttle.clone();
            let barrier = barrier.clone();
            workers.push(thread::spawn(move || {
                barrier.wait();
                throttle.allow_register_device(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)))
            }));
        }

        barrier.wait();
        let allowed = workers
            .into_iter()
            .map(|worker| worker.join().unwrap())
            .filter(|allowed| *allowed)
            .count();

        assert_eq!(allowed, 5);
    }

    #[test]
    fn retain_recent_removes_stale_keys() {
        let limiter = build_limiter::<String>(1, Duration::from_millis(10));

        assert_eq!(limiter.check_key(&"stale".to_string()), Ok(()));
        thread::sleep(Duration::from_millis(20));
        limiter.retain_recent();

        assert_eq!(limiter.len(), 0);
    }

    #[test]
    fn missing_pair_device_cache_expires() {
        let throttle = AuthThrottle::new(Duration::from_millis(10));
        let device_id = Uuid::new_v4();

        throttle.remember_missing_pair_device(device_id);
        assert!(throttle.is_known_missing_pair_device(device_id));

        thread::sleep(Duration::from_millis(20));

        assert!(!throttle.is_known_missing_pair_device(device_id));
    }
}
