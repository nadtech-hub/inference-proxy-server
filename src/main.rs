use actix_web::{App, HttpServer, web};
use reqwest::Client;
use std::collections::HashMap;
use std::env;
use std::io::{Error, ErrorKind};
use std::sync::atomic::AtomicU64;
use std::sync::{Arc, OnceLock, RwLock};
use std::time::Duration;

pub mod cache_manager;
pub mod forward_req;
pub mod models;
mod req_handler;
use models::AppCfg;
use req_handler::handle_req;

pub static REQUEST_MAP: OnceLock<RwLock<HashMap<String, Vec<String>>>> = OnceLock::new();
pub static REQ_COUNTER: OnceLock<AtomicU64> = OnceLock::new();
const EMBED_PATH: &str = "/embed";

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // read and parse env vars
    dotenvy::dotenv().map_err(|e| Error::new(ErrorKind::Other, e))?;
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    let max_wait_time =
        env::var("MAX_WAIT_TIME_SEC").map_err(|e| Error::new(ErrorKind::Other, e))?;
    let max_wait_time_parsed = max_wait_time
        .parse::<u64>()
        .map_err(|e| Error::new(ErrorKind::Other, e))?;
    let max_batch_size = env::var("MAX_BATCH_SIZE").map_err(|e| Error::new(ErrorKind::Other, e))?;
    let max_batch_size_parsed = max_batch_size
        .parse::<u64>()
        .map_err(|e| Error::new(ErrorKind::Other, e))?;
    let mut inference_service_url =
        env::var("INFERENCE_SERVICE_URI").map_err(|e| Error::new(ErrorKind::Other, e))?;
    inference_service_url = inference_service_url + EMBED_PATH;

    // instantiate job scheduler and request map
    let m = HashMap::<String, Vec<String>>::new();
    let req_rw_lock = RwLock::new(m);
    // encapsulate map and scheduler in cell
    REQUEST_MAP
        .set(req_rw_lock)
        .map_err(|_| Error::new(ErrorKind::Other, "Couldn't instantiate a map of requests"))?;

    let sz_rw_lock = AtomicU64::new(0);
    REQ_COUNTER
        .set(sz_rw_lock)
        .map_err(|_| Error::new(ErrorKind::Other, "Couldn't instantiate req counter"))?;

    // start server
    HttpServer::new(move || {
        let app_cfg = AppCfg {
            max_wait_time: Arc::new(max_wait_time_parsed),
            max_batch_size: Arc::new(max_batch_size_parsed),
            inference_service_url: inference_service_url.clone(),
        };
        App::new()
            .app_data(web::Data::new(Client::default()))
            .app_data(web::Data::new(app_cfg))
            .service(web::resource(EMBED_PATH).route(web::post().to(handle_req)))
    })
    .workers(1)
    .keep_alive(Duration::from_secs(0))
    .max_connections(max_batch_size_parsed as usize)
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
