use crate::models::AppCfg;
use reqwest::{Client, Method};
use std::{
    io::{Error, ErrorKind},
    sync::{Arc, RwLock}, time::Instant,
};

pub async fn forward(
    ip: &str,
    app_cfg: AppCfg,
    input: String,
    client: &Client,
    outputs: Arc<RwLock<Vec<Vec<f64>>>>,
) -> std::io::Result<()> {
    let url = app_cfg.inference_service_url.clone();
    log::info!("URL {url:?}");
    let mut forwarded_req = client.request(Method::POST, url);

    forwarded_req = forwarded_req.header("x-forwarded-for", ip);
    forwarded_req = forwarded_req.header("Content-Type", "application/json");

    let payload = serde_json::json!({
      "inputs": input,
      "normalize": true,
      "truncate": false,
      "truncation_direction": "Right"
    });
    forwarded_req = forwarded_req.json(&payload);
    
    let now = Instant::now();
    let res = forwarded_req
        .send()
        .await
        .map_err(|e| Error::new(ErrorKind::Other, e.to_string()))?;
    let mut emb = res
        .json::<Vec<Vec<f64>>>()
        .await
        .map_err(|e| Error::new(ErrorKind::Other, e))?;
    let mut outputs_lock = outputs
        .write()
        .map_err(|e| Error::new(ErrorKind::Other, e.to_string()))?;
    outputs_lock.append(&mut emb);
    let elapsed = now.elapsed();
    log::info!("Request to inference service from {ip} executed in {elapsed:?} time");
    Ok(())
}
