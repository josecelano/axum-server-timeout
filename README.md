# Example axum-server with timeouts

[![Testing](https://github.com/josecelano/axum-server-timeout/actions/workflows/testing.yaml/badge.svg)](https://github.com/josecelano/axum-server-timeout/actions/workflows/testing.yaml)

Related to:

- <https://github.com/programatik29/axum-server/issues/116>
- <https://github.com/torrust/torrust-tracker/issues/612>
- <https://github.com/torrust/torrust-tracker/issues/613>
- <https://github.com/tokio-rs/axum/discussions/1383>
- <https://github.com/hyperium/hyper/issues/3178>
- <https://docs.rs/actix-web/latest/actix_web/struct.HttpServer.html#method.client_request_timeout>
- <https://docs.rs/actix-web/latest/actix_web/struct.HttpServer.html#method.tls_handshake_timeout>
- <https://gist.github.com/programatik29/36d371c657392fd7f322e7342957b6d1>

I'm trying to setup [axum-server](https://github.com/programatik29/axum-server/) with a timeout.

The timeout I want to set is for the time the server is waiting for a request after opening a connection.

This example set a timeout in the server for sending the headers, by simulating a slow client.

The goal is to just open a connection without sending any requests. The server should close the connection after a while.

Run the server:

```output
cargo run --example server
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.02s
     Running `target/debug/examples/server`
Starting server on: http://127.0.0.1:3000 ...
Server bound to address: http://127.0.0.1:3000
New request ...
New request ...
New request ...
```

Run the client:

```output
$ cargo run --example client
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.02s
     Running `target/debug/examples/client`
Connected to the server: http://127.0.0.1:3000!
Client read timeout: Ok(Some(120s))
Client write timeout: Ok(Some(120s))
New request ...
Response OK: 130 bytes
HTTP/1.1 200 OK
content-type: text/plain; charset=utf-8
content-length: 13
date: Wed, 17 Apr 2024 11:07:41 GMT

Hello. world!
New slow request ...
thread 'main' panicked at examples/client.rs:63:14:
Failed to write to stream: Os { code: 104, kind: ConnectionReset, message: "Connection reset by peer" }
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

You can comment this line in the client:

```rust
send_request_slowly(&mut stream, request.as_bytes()).await;
```

And you will see how the connection is still open after 15 seconds without sending any requests.
