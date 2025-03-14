use std::net::SocketAddr;
use std::convert::Infallible;
use hyper::server::conn::http1;
use hyper_util::rt::TokioIo;
use hyper::service::service_fn;
use hyper::{Request, Response, Method, StatusCode};
use http_body_util::Full;
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

async fn proxy_handler(req: Request<hyper::body::Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {
    match *req.method() {
        Method::CONNECT => {
            // For now, we'll return a 501 Not Implemented for CONNECT requests
            Ok(Response::builder()
                .status(StatusCode::NOT_IMPLEMENTED)
                .body(Full::new(Bytes::from("CONNECT method not implemented")))
                .unwrap())
        }
        _ => {
            // For regular HTTP requests, we'll return a 200 OK with the requested URI
            let uri = req.uri().to_string();
            Ok(Response::builder()
                .status(StatusCode::OK)
                .body(Full::new(Bytes::from(format!("Received request for: {}", uri))))
                .unwrap())
        }
    }
} 