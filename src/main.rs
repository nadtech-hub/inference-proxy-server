use actix_web::{App, HttpServer, web};
use std::collections::HashMap;
use std::env;
use std::io::{Error, ErrorKind};
use std::sync::{Arc, OnceLock, RwLock};
use std::time::Duration;
use tokio_cron_scheduler::JobScheduler;
use reqwest::Client;

pub mod app_cfg;
pub mod cache_manager;
pub mod forward_req;
//mod body_validator;
mod req_handler;
use app_cfg::AppCfg;
use req_handler::handle_req;

pub static REQUEST_MAP: OnceLock<RwLock<HashMap<String, Vec<String>>>> =
    OnceLock::new();
pub static JOB_SCHEDULER: OnceLock<RwLock<JobScheduler>> = OnceLock::new();
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
    let scheduler = JobScheduler::new()
        .await
        .map_err(|e| Error::new(ErrorKind::Other, e))?;
    let m = HashMap::<String, Vec<String>>::new();
    let req_rw_lock = RwLock::new(m);
    let sched_rw_lock = RwLock::new(scheduler);

    // encapsulate map and scheduler in cell
    REQUEST_MAP
        .set(req_rw_lock)
        .map_err(|_| Error::new(ErrorKind::Other, "Couldn't instantiate a map of requests"))?;
    JOB_SCHEDULER
        .set(sched_rw_lock)
        .map_err(|_| Error::new(ErrorKind::Other, "Couldn't instantiate a job scheduler"))?;

    // start server
    HttpServer::new(move || {
        let app_cfg = AppCfg {
            max_wait_time: Arc::new(max_wait_time_parsed),
            max_batch_size: Arc::new(max_batch_size_parsed),
            inference_service_url: inference_service_url.clone(),
        };
        App::new()
            .app_data(web::Data::new(Client::default()))
            .service(
                web::resource("/")
                    .app_data(web::Data::new(app_cfg))
                    .route(web::post().to(handle_req)),
            )
    })
    .workers(2)
    .keep_alive(Duration::from_secs(0))
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
