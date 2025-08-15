use crate::REQ_COUNTER;
use crate::cache_manager::{add_inputs, get_inputs};
use crate::forward_req::forward;
use crate::models::{AppCfg, InputBody, OutputBody};
use actix_web::{
    HttpRequest, HttpResponse, dev::PeerAddr, http::StatusCode as ActixStatusCode, web,
};
use futures::future::join_all;
use reqwest::{Client, StatusCode};
use std::str::FromStr;
use std::sync::atomic::Ordering;
use std::sync::{Arc, RwLock};
use std::time::Instant;
use tokio::time::sleep;

pub async fn handle_req(
    _req: HttpRequest,
    payload: web::Json<InputBody>,
    peer_addr: Option<PeerAddr>,
    http_client: web::Data<Client>,
    app_cfg: web::Data<AppCfg>,
) -> std::io::Result<HttpResponse> {
    let max_batch_size = *app_cfg.max_batch_size;
    let cnt_lock = REQ_COUNTER.get().unwrap();
    let curr_val = cnt_lock.fetch_add(1, Ordering::Release) + 1;

    let mut status_code =
        ActixStatusCode::from_u16(StatusCode::TOO_MANY_REQUESTS.as_u16()).unwrap();
    if curr_val > max_batch_size {
        cnt_lock.store(0, Ordering::Release);
        let resp = HttpResponse::build(status_code).finish();
        return Ok(resp);
    }

    let now = Instant::now();
    let inputs = payload.inputs.clone();
    let app_cfg = app_cfg.get_ref().clone();
    let delay = (*app_cfg.max_wait_time).clone();

    status_code = ActixStatusCode::from_u16(StatusCode::BAD_REQUEST.as_u16()).unwrap();
    if inputs.len() == 0 {
        let resp = HttpResponse::build(status_code).finish();
        return Ok(resp);
    }

    let ip = peer_addr.clone().unwrap().0.ip().to_string();
    let res = add_inputs(String::from_str(ip.as_str()).unwrap(), inputs);

    if res.0 > 0 && res.0 == res.1 {
        let handle = tokio::spawn(async move {
            let dur = tokio::time::Duration::from_secs(delay);
            sleep(dur).await;
            let inputs = get_inputs(ip.as_str());
            let mut handles = Vec::new();
            let outputs = Arc::new(RwLock::new(Vec::<Vec<f64>>::new()));
            //let outputs_cloned = outputs.clone();

            for input in inputs.into_iter() {
                let ip = ip.clone();
                let app_cfg_cloned = app_cfg.clone();
                let http_client_cloned = http_client.as_ref().clone();
                let outputs_cloned = outputs.clone();
                let i = input.clone();

                let handle = tokio::spawn(async move {
                    match forward(
                        ip.as_str(),
                        app_cfg_cloned,
                        i,
                        &http_client_cloned,
                        outputs_cloned,
                    )
                    .await
                    {
                        Ok(_) => {
                            log::info!("Input sent successfully")
                        }
                        Err(err) => {
                            log::error!("Failed to sent input. Reason: {err:?}")
                        }
                    }
                });
                handles.push(handle);
            }
            let _ = join_all(handles).await;
            let cnt_lock = REQ_COUNTER.get().unwrap();
            cnt_lock.store(0, Ordering::Release);
            outputs
        });
        let outputs_res = handle.await?;
        let outputs = outputs_res.read().unwrap();
        status_code = ActixStatusCode::from_u16(StatusCode::OK.as_u16()).unwrap();
        let resp = HttpResponse::build(status_code).json(OutputBody {
            outputs: outputs.to_vec(),
        });
        return Ok(resp);
    } else if res.0 == 0 && res.1 == 0 {
        status_code = ActixStatusCode::from_u16(StatusCode::CONFLICT.as_u16()).unwrap();
        let res = HttpResponse::build(status_code).finish();
        return Ok(res);
    }

    status_code = ActixStatusCode::from_u16(StatusCode::ACCEPTED.as_u16()).unwrap();
    let resp = HttpResponse::build(status_code).finish();
    let elapsed = now.elapsed();
    log::info!("Request from {ip:?} executed in {elapsed:?} time");
    Ok(resp)
}
