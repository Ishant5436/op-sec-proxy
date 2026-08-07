use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{body::Bytes, Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use http_body_util::{Full, BodyExt};
use serde_json::Value;

use crate::rpc_client::RpcForwarder;

const INTERNAL_ERROR_JSON: &str = r#"{"jsonrpc":"2.0","id":null,"error":{"code":-32603,"message":"Internal server error"}}"#;

#[derive(Clone)]
struct AppState {
    forwarder: RpcForwarder,
    upstream_url: String,
}

pub async fn run_server(port: u16, upstream_url: String) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = TcpListener::bind(addr).await?;
    println!("AESI Proxy listening on http://{}", addr);

    let state = AppState {
        forwarder: RpcForwarder::new(upstream_url.clone()),
        upstream_url,
    };

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);
        let state_clone = state.clone();

        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(io, service_fn(move |req| handle_request(req, state_clone.clone())))
                .await
            {
                eprintln!("Error serving connection: {:?}", err);
            }
        });
    }
}

async fn handle_request(
    req: Request<hyper::body::Incoming>,
    state: AppState,
) -> Result<Response<Full<Bytes>>, hyper::Error> {
    if req.method() == hyper::Method::POST {
        let body_bytes = match req.into_body().collect().await {
            Ok(collected) => collected.to_bytes(),
            Err(_) => {
                let mut bad_req = Response::new(Full::new(Bytes::from("Bad Request")));
                *bad_req.status_mut() = StatusCode::BAD_REQUEST;
                return Ok(bad_req);
            }
        };
        
        let payload: Value = match serde_json::from_slice(&body_bytes) {
            Ok(v) => v,
            Err(_) => {
                let mut bad_req = Response::new(Full::new(Bytes::from("Bad Request")));
                *bad_req.status_mut() = StatusCode::BAD_REQUEST;
                return Ok(bad_req);
            }
        };

        // Run AESI Security Heuristics inside a blocking thread
        // to protect the async Tokio runtime from synchronous RPC I/O in fork_db.
        let upstream = state.upstream_url.clone();
        let payload_for_sim = payload.clone();
        let check_result = tokio::task::spawn_blocking(move || {
            crate::interceptor::check_payload(&payload_for_sim, &upstream)
        }).await;

        match check_result {
            Ok(Err(err_resp)) => {
                // Transaction blocked by heuristics
                let resp_str = serde_json::to_string(&err_resp).unwrap_or_else(|_|
                    INTERNAL_ERROR_JSON.to_string()
                );
                return Ok(Response::new(Full::new(Bytes::from(resp_str))));
            }
            Err(join_err) => {
                // spawn_blocking task panicked or was cancelled
                eprintln!("Simulation task failed: {}", join_err);
                let mut err_resp = Response::new(Full::new(Bytes::from(INTERNAL_ERROR_JSON)));
                *err_resp.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                return Ok(err_resp);
            }
            Ok(Ok(())) => { /* Passed heuristics, continue to forward */ }
        }

        match state.forwarder.forward(payload).await {
            Ok(resp_val) => {
                let resp_str = serde_json::to_string(&resp_val).unwrap_or_else(|_|
                    INTERNAL_ERROR_JSON.to_string()
                );
                Ok(Response::new(Full::new(Bytes::from(resp_str))))
            }
            Err(e) => {
                eprintln!("Forwarding error: {}", e);
                let mut err_resp = Response::new(Full::new(Bytes::from(INTERNAL_ERROR_JSON)));
                *err_resp.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                Ok(err_resp)
            }
        }
    } else {
        let mut not_found = Response::new(Full::new(Bytes::from("Not Found")));
        *not_found.status_mut() = StatusCode::NOT_FOUND;
        Ok(not_found)
    }
}
