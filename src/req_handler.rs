use crate::JOB_SCHEDULER;
use crate::app_cfg::AppCfg;
use crate::cache_manager::{add_inputs, get_inputs};
use crate::forward_req::forward;
use actix_web::{
    HttpRequest, HttpResponse, dev::PeerAddr, http::StatusCode as ActixStatusCode, web,
};
use reqwest::{Client, StatusCode};
use std::str::FromStr;
use std::time::Instant;
use std::{
    io::{Error, ErrorKind},
    sync::{Arc, RwLock},
    time::Duration,
};
use tokio_cron_scheduler::Job;

pub async fn handle_req(
    req: HttpRequest,
    payload: web::Json<Vec<String>>,
    peer_addr: Option<PeerAddr>,
    http_client: web::Data<Client>,
) -> std::io::Result<HttpResponse> {
    let now = Instant::now();
    let inputs = payload.0.clone();
    let app_cfg_inner = req.app_data::<AppCfg>().unwrap();
    let delay = (*app_cfg_inner.max_wait_time).clone();

    let mut status_code = ActixStatusCode::from_u16(StatusCode::BAD_REQUEST.as_u16()).unwrap();
    if inputs.len() == 0 {
        let resp = HttpResponse::build(status_code).finish();
        return Ok(resp);
    }

    let ip = peer_addr.clone().unwrap().0.ip().to_string();
    // let peer_addr = peer_addr.clone().unwrap();
    // let ip_addr = peer_addr.0.ip().clone();
    // let ip = Box::new(ip_addr.to_string()).into_boxed_str();

    let res = add_inputs(String::from_str(ip.as_str()).unwrap(), inputs);
    if res.0 == 0 {
        status_code = ActixStatusCode::from_u16(StatusCode::CONFLICT.as_u16()).unwrap();
        let res = HttpResponse::build(status_code).finish();
        return Ok(res);
    } else if res.0 == res.1 {
        let sched_cell = JOB_SCHEDULER.get().unwrap();
        let outputs = Arc::new(RwLock::new(Vec::<u64>::new()));
        let inputs = get_inputs(ip.as_str());
        let _res = match sched_cell.write() {
            Ok(sched) => {
                for input in inputs.into_iter() {
                    let ip = ip.clone();
                    let app_cfg_cloned = app_cfg_inner.clone();
                    let http_client_cloned = http_client.as_ref().clone();
                    let outputs_cloned = outputs.clone();

                    let locked_job =
                        Job::new_one_shot_async(Duration::from_secs(delay), move |_uuid, _l| {
                            Box::pin({
                                let ip = ip.clone().to_string();
                                let ac = app_cfg_cloned.clone();
                                let hc = http_client_cloned.clone();
                                let o = outputs_cloned.clone();
                                let i = input.clone();

                                async move {
                                    match forward(ip.as_str(), ac, i, &hc, o).await {
                                        Ok(_) => {
                                            log::info!("Input sent successfully")
                                        }
                                        Err(err) => {
                                            log::error!("Failed to sent input. Reason: {err}")
                                        }
                                    }
                                }
                            })
                        })
                        .map_err(|e| Error::new(ErrorKind::Other, e))?;
                    let _ = sched.add(locked_job).await;
                }
            }
            Err(_err) => {
                status_code = ActixStatusCode::from_u16(StatusCode::CONFLICT.as_u16()).unwrap();
                let res = HttpResponse::build(status_code).finish();
                return Ok(res);
            }
        };
    }

    status_code = ActixStatusCode::from_u16(StatusCode::ACCEPTED.as_u16()).unwrap();
    let resp = HttpResponse::build(status_code).finish();
    let elapsed = now.elapsed();
    log::info!("Request from {ip:?} executed in {elapsed:?} time");
    Ok(resp)
}
