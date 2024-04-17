# Example Axum-server with timeouts

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

I've added other server implementations using other frameworks to also check their behavior.

## Client

There is a client that makes two requests and then waits some seconds.

- The first request is a normal `GET` request to test the connection to the server.
- The second request is a slow request to test server timeout. The timeout for sending the request headers.
- Finally, the client waits for some seconds to test if the server closes the connection.

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

You can comment the "slow" request line in the client:

```rust
// send_request_slowly(&mut stream, request.as_bytes()).await;
```

To check what happens if the client is idle without sending any requests.

## Axum-server

Run the server:

```output
cargo run --example axum_server
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.02s
     Running `target/debug/examples/server`
Starting server on: http://127.0.0.1:3000 ...
Server bound to address: http://127.0.0.1:3000
New request ...
```

## Rocket

Run the server:

```output
ROCKET_KEEP_ALIVE=1 ROCKET_PORT=3000 cargo run --example rocket_server
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.04s
     Running `target/debug/examples/rocket_server`
🔧 Configured for debug.
   >> address: 127.0.0.1
   >> port: 3000
   >> workers: 32
   >> max blocking threads: 512
   >> ident: Rocket
   >> IP header: X-Real-IP
   >> limits: bytes = 8KiB, data-form = 2MiB, file = 1MiB, form = 32KiB, json = 1MiB, msgpack = 1MiB, string = 8KiB
   >> temp dir: /tmp
   >> http/2: true
   >> keep-alive: 1s
   >> tls: disabled
   >> shutdown: ctrlc = true, force = true, signals = [SIGTERM], grace = 2s, mercy = 3s
   >> log level: normal
   >> cli colors: true
📬 Routes:
   >> (hello) GET /
📡 Fairings:
   >> Shield (liftoff, response, singleton)
🛡️ Shield:
   >> X-Frame-Options: SAMEORIGIN
   >> X-Content-Type-Options: nosniff
   >> Permissions-Policy: interest-cohort=()
🚀 Rocket has launched from http://127.0.0.1:3000
GET /:
   >> Matched: (hello) GET /
   >> Outcome: Success(200 OK)
   >> Response succeeded.
```
