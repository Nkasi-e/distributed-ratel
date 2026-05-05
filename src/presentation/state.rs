use std::sync::Arc;


use crate::application::policy::PolicyTable;
use crate::application::ports::RateLimiter;



#[derive(Clone)]
pub struct AppState {
    pub limiter: Arc<dyn RateLimiter>,
    pub policy: PolicyTable
}