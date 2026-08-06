use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{body::Bytes, Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use http_body_util::{Full, BodyExt};
use serde_json::Value;

use crate::rpc_client::RpcForwarder;

#[derive(Clone)]
struct AppState {
    forwarder: RpcForwarder,
}

pub async fn run_server(port: u16, upstream_url: String) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = TcpListener::bind(addr).await?;
    println!("AESI Proxy listening on http://{}", addr);

    let state = AppState {
        forwarder: RpcForwarder::new(upstream_url),
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

        // Run AESI Security Heuristics
        if let Err(err_resp) = crate::interceptor::check_payload(&payload) {
            let resp_str = serde_json::to_string(&err_resp).unwrap_or_default();
            return Ok(Response::new(Full::new(Bytes::from(resp_str))));
        }

        match state.forwarder.forward(payload).await {
            Ok(resp_val) => {
                let resp_str = serde_json::to_string(&resp_val).unwrap_or_default();
                Ok(Response::new(Full::new(Bytes::from(resp_str))))
            }
            Err(e) => {
                eprintln!("Forwarding error: {}", e);
                let mut err_resp = Response::new(Full::new(Bytes::from("Internal Server Error")));
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
