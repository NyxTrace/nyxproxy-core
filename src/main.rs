mod proxy;

use std::net::SocketAddr;
use proxy::HttpProxy;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Create proxy servers listening on both IPv4 and IPv6
    let addr_v4: SocketAddr = "0.0.0.0:8080".parse()?;
    let addr_v6: SocketAddr = "[::]:8080".parse()?;

    println!("Starting HTTP proxy servers...");
    
    // Spawn both servers
    let v4_handle = tokio::spawn(async move {
        if let Err(e) = HttpProxy::new(addr_v4).run().await {
            eprintln!("IPv4 proxy error: {}", e);
        }
    });

    let v6_handle = tokio::spawn(async move {
        if let Err(e) = HttpProxy::new(addr_v6).run().await {
            eprintln!("IPv6 proxy error: {}", e);
        }
    });

    // Wait for both servers
    tokio::try_join!(v4_handle, v6_handle)?;

    Ok(())
}
