use std::{sync::Arc};
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Clone)]
pub struct AppCfg {
    pub max_wait_time: Arc<u64>,
    pub max_batch_size: Arc<u64>,
    pub inference_service_url: String,
}

#[derive(Deserialize, Debug, Validate)]
pub struct InputBody {
    #[validate(length(min = 1))]
    pub inputs: Vec<String>
}

#[derive(Serialize, Debug, Validate)]
pub struct OutputBody {
    #[validate(length(min = 1))]
    pub outputs: Vec<Vec<f64>>
}
