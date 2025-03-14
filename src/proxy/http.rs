use std::net::SocketAddr;
use std::convert::Infallible;
use hyper::server::conn::http1;
use hyper_util::rt::TokioIo;
use hyper::service::service_fn;
use hyper::{Request, Response, Method, StatusCode, body::Incoming};
use hyper_util::client::legacy::{connect::HttpConnector, Client};
use http_body_util::{Full, BodyExt};
use bytes::Bytes;
use tokio::net::TcpListener;

pub struct HttpProxy {
    addr: SocketAddr,
}

impl HttpProxy {
    pub fn new(addr: SocketAddr) -> Self {
        Self { addr }
    }

    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let listener = TcpListener::bind(self.addr).await?;
        println!("Proxy server running on http://{}", self.addr);

        loop {
            let (stream, _) = listener.accept().await?;
            let io = TokioIo::new(stream);

            tokio::task::spawn(async move {
                if let Err(err) = http1::Builder::new()
                    .serve_connection(io, service_fn(proxy_handler))
                    .await
                {
                    eprintln!("Error serving connection: {}", err);
                }
            });
        }
    }
}

async fn proxy_handler(req: Request<Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {
    let client = Client::builder(hyper_util::rt::TokioExecutor::new()).build(HttpConnector::new());

    match *req.method() {
        Method::CONNECT => {
            Ok(Response::builder()
                .status(StatusCode::NOT_IMPLEMENTED)
                .body(Full::new(Bytes::from("CONNECT method not implemented")))
                .unwrap())
        }
        _ => {
            match client.request(req).await {
                Ok(response) => {
                    let (parts, body) = response.into_parts();
                    let bytes = body.collect().await.unwrap_or_default().to_bytes();
                    Ok(Response::from_parts(parts, Full::new(bytes)))
                }
                Err(e) => {
                    eprintln!("Error forwarding request: {}", e);
                    Ok(Response::builder()
                        .status(StatusCode::BAD_GATEWAY)
                        .body(Full::new(Bytes::from(format!("Error: {}", e))))
                        .unwrap())
                }
            }
        }
    }
} 