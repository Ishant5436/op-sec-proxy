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
    run_server_listener(listener, upstream_url).await
}

pub async fn run_server_listener(listener: TcpListener, upstream_url: String) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = listener.local_addr()?;
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

pub const MAX_REQUEST_BODY_SIZE: usize = 2 * 1024 * 1024; // 2 MB

fn build_json_response(body_str: String, status: StatusCode) -> Response<Full<Bytes>> {
    let mut resp = Response::new(Full::new(Bytes::from(body_str)));
    *resp.status_mut() = status;
    resp.headers_mut().insert("Content-Type", "application/json".parse().unwrap());
    resp.headers_mut().insert("Access-Control-Allow-Origin", "*".parse().unwrap());
    resp.headers_mut().insert("Access-Control-Allow-Methods", "POST, OPTIONS".parse().unwrap());
    resp.headers_mut().insert("Access-Control-Allow-Headers", "Content-Type".parse().unwrap());
    resp
}

async fn handle_request(
    req: Request<hyper::body::Incoming>,
    state: AppState,
) -> Result<Response<Full<Bytes>>, hyper::Error> {
    if req.method() == hyper::Method::OPTIONS {
        let mut preflight = Response::new(Full::new(Bytes::default()));
        *preflight.status_mut() = StatusCode::NO_CONTENT;
        preflight.headers_mut().insert("Access-Control-Allow-Origin", "*".parse().unwrap());
        preflight.headers_mut().insert("Access-Control-Allow-Methods", "POST, OPTIONS".parse().unwrap());
        preflight.headers_mut().insert("Access-Control-Allow-Headers", "Content-Type".parse().unwrap());
        return Ok(preflight);
    }

    if req.method() == hyper::Method::POST {
        // Fast-path Content-Length header validation to prevent buffering oversized requests
        if req
            .headers()
            .get(hyper::header::CONTENT_LENGTH)
            .and_then(|h| h.to_str().ok())
            .and_then(|s| s.parse::<usize>().ok())
            .is_some_and(|cl| cl > MAX_REQUEST_BODY_SIZE)
        {
            return Ok(build_json_response(
                r#"{"jsonrpc":"2.0","id":null,"error":{"code":-32600,"message":"Request body too large: max allowed is 2MB"}}"#.to_string(),
                StatusCode::PAYLOAD_TOO_LARGE,
            ));
        }

        // Bounded stream collection: enforce MAX_REQUEST_BODY_SIZE even for chunked / unannounced transfers
        let limited_body = http_body_util::Limited::new(req.into_body(), MAX_REQUEST_BODY_SIZE);
        let body_bytes = match limited_body.collect().await {
            Ok(collected) => collected.to_bytes(),
            Err(err) => {
                let is_overflow = err.downcast_ref::<http_body_util::LengthLimitError>().is_some();
                let (status, msg) = if is_overflow {
                    (
                        StatusCode::PAYLOAD_TOO_LARGE,
                        r#"{"jsonrpc":"2.0","id":null,"error":{"code":-32600,"message":"Request body too large: max allowed is 2MB"}}"#,
                    )
                } else {
                    (
                        StatusCode::BAD_REQUEST,
                        r#"{"jsonrpc":"2.0","id":null,"error":{"code":-32700,"message":"Parse error"}}"#,
                    )
                };
                return Ok(build_json_response(msg.to_string(), status));
            }
        };
        
        let payload: Value = match serde_json::from_slice(&body_bytes) {
            Ok(v) => v,
            Err(_) => {
                return Ok(build_json_response(
                    r#"{"jsonrpc":"2.0","id":null,"error":{"code":-32700,"message":"Parse error"}}"#.to_string(),
                    StatusCode::BAD_REQUEST
                ));
            }
        };

        // Run Security Heuristics inside a blocking thread
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
                return Ok(build_json_response(resp_str, StatusCode::OK));
            }
            Err(join_err) => {
                // spawn_blocking task panicked or was cancelled
                eprintln!("Simulation task failed: {}", join_err);
                return Ok(build_json_response(INTERNAL_ERROR_JSON.to_string(), StatusCode::INTERNAL_SERVER_ERROR));
            }
            Ok(Ok(())) => { /* Passed heuristics, continue to forward */ }
        }

        match state.forwarder.forward(payload).await {
            Ok(resp_val) => {
                let resp_str = serde_json::to_string(&resp_val).unwrap_or_else(|_|
                    INTERNAL_ERROR_JSON.to_string()
                );
                Ok(build_json_response(resp_str, StatusCode::OK))
            }
            Err(e) => {
                eprintln!("Forwarding error: {}", e);
                Ok(build_json_response(INTERNAL_ERROR_JSON.to_string(), StatusCode::INTERNAL_SERVER_ERROR))
            }
        }
    } else {
        Ok(build_json_response("Not Found".to_string(), StatusCode::NOT_FOUND))
    }
}
