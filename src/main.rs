use std::sync::Arc;

use axum::serve;
use distributed_ratel::application::memory_limiter::MemoryRateLimiter;
use distributed_ratel::application::policy::PolicyTable;
use distributed_ratel::application::ports::{MonotonicClock, RateLimitStore, RateLimiter};
use distributed_ratel::application::service::AllowService;
// use distributed_ratel::domain::key::RateLimitKey;
use distributed_ratel::infrastructure::clock::SystemClock;
use distributed_ratel::infrastructure::config::{AppConfig, BackendKind};
use distributed_ratel::infrastructure::redis_limiter::RedisRateLimiter;
use distributed_ratel::infrastructure::telemetry;
use distributed_ratel::presentation::{build_router, AppState};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    telemetry::init_tracing();

    let cfg = AppConfig::load()?;
    let policy = PolicyTable::try_from_config(cfg.rate_limit)?;
    //    let clock: Arc<dyn MonotonicClock> = Arc::new(SystemClock::new());
    let store: Arc<dyn RateLimitStore> = match cfg.storage.backend {
        BackendKind::Memory => {
            let clock: Arc<dyn MonotonicClock> = Arc::new(SystemClock::new());
            Arc::new(MemoryRateLimiter::new(policy.clone(), clock))
        }

        BackendKind::Redis => Arc::new(
            RedisRateLimiter::new(cfg.storage.redis, cfg.storage.fallback_strategy, policy.clone()).await?,
        ),
    };
    //    let limiter = MemoryRateLimiter::new(policy, Arc::clone(&clock));
    //    let service = AllowService::memory(limiter);
    let service = AllowService::new(store);
    let limiter: Arc<dyn RateLimiter> = Arc::new(service);

    let state = AppState {
        limiter,
        policy
    };

    let app = build_router(state);
    let addr = "0.0.0.0:8080";
    let listener = TcpListener::bind(addr).await?;
    tracing::info!("listening on http://{addr}");
    serve(listener, app).await?;

    // let key = RateLimitKey::UserId("user-1".into());
    // let ok = service.allow(&key, 1).await?;
    // tracing::info!(%ok, ?key, "allow check");

    Ok(())
}
