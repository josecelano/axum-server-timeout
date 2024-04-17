use std::{
    io::{Read, Write},
    net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream},
    time::Duration,
};

#[tokio::main]
async fn main() {
    let server_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 3000);

    // Open a TCP connection to the server
    if let Ok(mut stream) = TcpStream::connect(server_addr) {
        println!("Connected to the server: http://{server_addr}!"); // DevSkim: ignore DS137138

        // Set long timeouts for client to avoid timeouts on the client side.
        stream
            .set_read_timeout(Some(Duration::from_secs(120)))
            .expect("Set read timeout");
        stream
            .set_write_timeout(Some(Duration::from_secs(120)))
            .expect("Set write timeout");

        // Sleep for some duration to observe the server's behavior.
        // The server should close the connection when no requests are sent.
        println!("Sleeping 15 seconds ...");
        std::thread::sleep(Duration::from_secs(15));

        println!("Client read timeout: {:?}", stream.read_timeout());
        println!("Client write timeout: {:?}", stream.write_timeout());

        let request = "GET / HTTP/1.1\r\nHost: localhost\r\n\r\n"; // Extra line break is  needed.

        // Send an HTTP GET request to the root endpoint (fast request)
        // This is only to confirm the client and server are working fine.
        send_request(&mut stream, request.as_bytes());

        // Send the same request taking longer than the server timeout.
        // The client should panic because the server closes the connection.
        send_request_slowly(&mut stream, request.as_bytes()).await;

        println!("Client stream: {stream:?}");
    } else {
        println!("Couldn't connect to server...");
    }
}

fn send_request(stream: &mut TcpStream, request: &[u8]) {
    println!("New request ...");

    stream
        .write_all(request)
        .expect("Failed to write to stream");

    read_response(stream);
}

async fn send_request_slowly(stream: &mut TcpStream, request: &[u8]) {
    println!("New slow request ...");

    for &byte in request {
        stream
            .write_all(&[byte])
            .expect("Failed to write to stream");

        // Sleep for a short duration between bytes
        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    read_response(stream);
}

fn read_response(stream: &mut TcpStream) {
    let mut buffer = vec![0; 1_048_576];
    let result = stream.read(&mut buffer);

    match result {
        Ok(size) => println!(
            "Response OK: {size:#?} bytes\n{}",
            String::from_utf8_lossy(&buffer)
        ),
        Err(err) => println!("Response Error: {err:#?}"),
    }
}
