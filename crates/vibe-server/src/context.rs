use std::sync::Arc;

use tokio_util::sync::CancellationToken;

use crate::{
    auth::AuthService,
    config::Config,
    events::EventRouter,
    machines::RpcRegistry,
    monitoring::Metrics,
    presence::PresenceService,
    storage::{db::Database, files::FileStorage, redis::RedisStore},
    version::VersionInfo,
};

#[derive(Clone)]
pub struct AppContext(Arc<AppContextInner>);

struct AppContextInner {
    config: Config,
    version: VersionInfo,
    db: Database,
    redis: RedisStore,
    files: FileStorage,
    auth: AuthService,
    events: EventRouter,
    rpc: RpcRegistry,
    presence: PresenceService,
    metrics: Metrics,
    shutdown: CancellationToken,
}

impl Drop for AppContextInner {
    fn drop(&mut self) {
        self.shutdown.cancel();
    }
}

impl AppContext {
    pub fn new(config: Config) -> Self {
        let shutdown = CancellationToken::new();
        let db = Database::default();
        let redis = RedisStore::default();
        let files = FileStorage::new(&config);
        let events = EventRouter::default();
        let rpc = RpcRegistry::default();
        let metrics = Metrics::default();
        let auth = AuthService::new(db.clone(), redis.clone(), config.master_secret.clone());
        let presence =
            PresenceService::new(db.clone(), redis.clone(), events.clone(), shutdown.clone());
        presence.spawn_background_tasks();

        Self(Arc::new(AppContextInner {
            version: VersionInfo::current(&config),
            config,
            db,
            redis,
            files,
            auth,
            events,
            rpc,
            presence,
            metrics,
            shutdown,
        }))
    }

    pub fn config(&self) -> &Config {
        &self.0.config
    }

    pub fn version(&self) -> &VersionInfo {
        &self.0.version
    }

    pub fn db(&self) -> &Database {
        &self.0.db
    }

    pub fn redis(&self) -> &RedisStore {
        &self.0.redis
    }

    pub fn files(&self) -> &FileStorage {
        &self.0.files
    }

    pub fn auth(&self) -> &AuthService {
        &self.0.auth
    }

    pub fn events(&self) -> &EventRouter {
        &self.0.events
    }

    pub fn presence(&self) -> &PresenceService {
        &self.0.presence
    }

    pub fn rpc(&self) -> &RpcRegistry {
        &self.0.rpc
    }

    pub fn metrics(&self) -> &Metrics {
        &self.0.metrics
    }

    pub async fn shutdown(&self) {
        self.0.shutdown.cancel();
        self.0.presence.flush_pending().await;
    }
}
