use axum_server_timeout::server;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

#[tokio::main]
async fn main() {
    let server_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 3000);

    println!("Starting server on: http://{server_addr} ..."); // DevSkim: ignore DS137138

    server::start(&server_addr).await;
}
