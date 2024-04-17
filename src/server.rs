use axum::routing::get;
use std::net::SocketAddr;
use std::time::Duration;

use axum::Router;
use hyper_util::rt::TokioTimer;

/// # Panics
///
/// Will panic if it can get the local server address
pub async fn start(bind_to: &SocketAddr) {
    let socket =
        std::net::TcpListener::bind(bind_to).expect("Could not bind tcp_listener to address.");

    let server_address = socket
        .local_addr()
        .expect("Could not get local_addr from tcp_listener.");

    println!("Server bound to address: http://{server_address}"); // DevSkim: ignore DS137138

    let mut server = axum_server::from_tcp(socket);

    server.http_builder().http1().timer(TokioTimer::new());
    server.http_builder().http2().timer(TokioTimer::new());

    server
        .http_builder()
        .http1()
        .header_read_timeout(Duration::from_secs(1));
    server
        .http_builder()
        .http2()
        .keep_alive_timeout(Duration::from_secs(1))
        .keep_alive_interval(Duration::from_secs(1));

    let app = Router::new().route("/", get(handler));

    server
        .serve(app.into_make_service_with_connect_info::<std::net::SocketAddr>())
        .await
        .expect("Axum server crashed.");
}

pub async fn handler() -> String {
    println!("New request ...");
    "Hello. world!".to_string()
}
