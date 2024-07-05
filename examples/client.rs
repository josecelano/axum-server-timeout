use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    time::Duration,
};

use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

#[tokio::main]
async fn main() {
    let server_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 3000);

    // Open a TCP connection to the server
    if let Ok(mut stream) = TcpStream::connect(server_addr).await {
        println!("Connected to the server: http://{server_addr}!"); // DevSkim: ignore DS137138

        println!("Sleeping 15 seconds without sending any requests ...");
        std::thread::sleep(Duration::from_secs(15));

        let request = "GET / HTTP/1.1\r\nHost: localhost\r\n\r\n"; // Extra line break is needed.

        // Send an HTTP GET request to the root endpoint (fast request)
        // This is only to confirm the client and server are working fine.
        send_request(&mut stream, request.as_bytes()).await.unwrap();

        stream.take_error().expect("No error was expected...");

        // Send the same request taking longer than the server timeout.
        // The client should panic because the server closes the connection.
        send_request_slowly(&mut stream, request.as_bytes())
            .await
            .unwrap();

        println!("Client stream: {stream:?}");
    } else {
        println!("Couldn't connect to server...");
    }
}

async fn send_request(stream: &mut TcpStream, request: &[u8]) -> Result<(), std::io::Error> {
    print!("Send request ... ");

    stream.write_all(request).await?;
    stream.flush().await?;

    read_response(stream).await;

    Ok(())
}

async fn send_request_slowly(stream: &mut TcpStream, request: &[u8]) -> Result<(), std::io::Error> {
    print!("Send slow request ...");

    for &byte in request {
        stream.write_all(&[byte]).await?;
        stream.flush().await?;

        // Sleep for a short duration between bytes
        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    read_response(stream).await;

    Ok(())
}

async fn read_response(stream: &mut TcpStream) {
    let mut buffer = vec![0; 1_048_576];

    let result = stream.read(&mut buffer).await;

    match result {
        Ok(size) => {
            if size != 0 {
                println!(
                    "response size: {size:#?} bytes\n{}",
                    String::from_utf8_lossy(&buffer)
                );
            } else {
                println!("no response");
            }
        }
        Err(err) => println!("Error reading response buffer: {err:#?}"),
    }
}
