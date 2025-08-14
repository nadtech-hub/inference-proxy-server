use std::{sync::Arc};

#[derive(Clone)]
pub struct AppCfg {
    pub max_wait_time: Arc<u64>,
    pub max_batch_size: Arc<u64>,
    pub inference_service_url: String,
}
