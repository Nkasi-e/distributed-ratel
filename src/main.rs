use std::sync::Arc;

use distributed_ratel::application::memory_limiter::MemoryRateLimiter;
use distributed_ratel::application::policy::PolicyTable;
use distributed_ratel::application::ports::{RateLimiter, MonotonicClock};
use distributed_ratel::application::service::AllowService;
use distributed_ratel::domain::key::RateLimitKey;
use distributed_ratel::infrastructure::clock::SystemClock;
use distributed_ratel::infrastructure::config::AppConfig;
use distributed_ratel::infrastructure::telemetry;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
   telemetry::init_tracing();

   let cfg = AppConfig::load()?;
   let policy = PolicyTable::from_config(cfg.rate_limit);
   let clock: Arc<dyn MonotonicClock> = Arc::new(SystemClock::new());
   let limiter = MemoryRateLimiter::new(policy, Arc::clone(&clock));
   let service = AllowService::memory(limiter);

   let key = RateLimitKey::UserId("user-1".into());
   let ok = service.allow(&key, 1).await?;
   tracing::info!(%ok, ?key, "allow check");

   Ok(())
}
