# Example Axum-server with timeouts

[![Testing](https://github.com/josecelano/axum-server-timeout/actions/workflows/testing.yaml/badge.svg)](https://github.com/josecelano/axum-server-timeout/actions/workflows/testing.yaml)

Related to:

- <https://github.com/programatik29/axum-server/issues/116>
- <https://github.com/torrust/torrust-tracker/issues/2776>
- <https://github.com/torrust/torrust-tracker/issues/612>
- <https://github.com/torrust/torrust-tracker/issues/613>
- <https://github.com/tokio-rs/axum/discussions/1383>
- <https://github.com/tokio-rs/axum/discussions/2716>
- <https://github.com/hyperium/hyper/issues/3178>
- <https://github.com/actix/actix-web/discussions/3337>
- <https://docs.rs/actix-web/latest/actix_web/struct.HttpServer.html#method.client_request_timeout>
- <https://docs.rs/actix-web/latest/actix_web/struct.HttpServer.html#method.tls_handshake_timeout>
- <https://gist.github.com/programatik29/36d371c657392fd7f322e7342957b6d1>
- <https://github.com/rwf2/Rocket/discussions/2774>
- <https://github.com/rwf2/Rocket/issues/1405>
- <https://github.com/hyperium/hyper/issues/1628>
- <https://hexadix.com/slowloris-dos-attack-mitigation-nginx-web-server/>
- <https://www.acunetix.com/blog/articles/slow-http-dos-attacks-mitigate-apache-http-server/>

I'm trying to setup [axum-server](https://github.com/programatik29/axum-server/) with a timeout.

The timeout I want to set is for the time the server is waiting for a request after opening a connection.

This example set a timeout in the server for sending the headers, by simulating a slow client.

The goal is to just open a connection without sending any requests. The server should close the connection after a while.

I've added other server implementations using other frameworks to also check their behavior.

**Update (2026-11):** fixed in `axum-server` 0.9.0, which adds
`Server::first_request_timeout`; see [Axum-server 0.9](#axum-server-09). The history below describes
`axum-server` 0.6 to 0.8, where it only worked for [ActixWeb](https://actix.rs/) and with a copy of a gist.

ActixWeb returns a `408 Request Timeout` for the first request and panics for the second one.

```output
Connected to the server: http://127.0.0.1:3000!
Sleeping 15 seconds ...
Client read timeout: Ok(Some(120s))
Client write timeout: Ok(Some(120s))
New request ...
Response OK: 107 bytes
HTTP/1.1 408 Request Timeout
content-length: 0
connection: close
date: Wed, 17 Apr 2024 17:18:43 GMT


New request ...
thread 'main' panicked at examples/client.rs:53:10:
Failed to write to stream: Os { code: 32, kind: BrokenPipe, message: "Broken pipe" }
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

## Client

There is a client example that waits some seconds and then makes two requests.

- First, the client waits for some seconds to test if the server closes the connection.
- The first request is a normal `GET` request to test the connection to the server.
- The second request is a slow request to test server timeout. The timeout for sending the request headers.

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
Starting server on: http://127.0.0.1:3000 ...
Server bound to address: http://127.0.0.1:3000
New request ...
```

With `axum-server` 0.6 to 0.8, the patch proposed [here](https://gist.github.com/programatik29/36d371c657392fd7f322e7342957b6d1) seemed to work partially. This repository used it until commit `eb383ef` (`src/server/slowloris.rs`, now removed). It closes the connection but it does not return a `408 Request Timeout` response.

The client output with ActixWeb server:

```output
Connected to the server: http://127.0.0.1:3000!
Sleeping 15 seconds without sending any requests ...
Send request ... response size: 107 bytes
HTTP/1.1 408 Request Timeout
content-length: 0
connection: close
date: Thu, 18 Apr 2024 15:26:51 GMT

thread 'main' panicked at examples/client.rs:43:14:
called `Result::unwrap()` on an `Err` value: Os { code: 32, kind: BrokenPipe, message: "Broken pipe" }
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

The client output with Axum-server:

```output
Connected to the server: http://127.0.0.1:3000!
Sleeping 15 seconds without sending any requests ...
Send request ... response size: 130 bytes
HTTP/1.1 200 OK
content-type: text/plain; charset=utf-8
content-length: 13
date: Fri, 05 Jul 2024 12:26:57 GMT

Hello. world!
thread 'main' panicked at examples/client.rs:33:14:
called `Result::unwrap()` on an `Err` value: Os { code: 104, kind: ConnectionReset, message: "Connection reset by peer" }
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

The client output with Axum-server 0.6 to 0.8 and the gist patch (`TimeoutAcceptor`, removed; see [Axum-server 0.9](#axum-server-09)):

```output
Connected to the server: http://127.0.0.1:3000!
Sleeping 15 seconds without sending any requests ...
Send request ... no response
thread 'main' panicked at examples/client.rs:45:14:
called `Result::unwrap()` on an `Err` value: Os { code: 32, kind: BrokenPipe, message: "Broken pipe" }
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

## Axum-server 0.9

`axum-server` 0.9.0 (2026-11-06) installs a Tokio timer by default and adds
[`Server::first_request_timeout`](https://docs.rs/axum-server/0.9.0/axum_server/struct.Server.html#method.first_request_timeout),
which closes a connection that has not sent a complete first request in time. The server in this repository now uses it
(see `src/server/mod.rs`) and no longer needs the gist's `TimeoutAcceptor`, which was removed.

Result with the same client (silent for 15 seconds, then a request):

```output
Connected to the server: http://127.0.0.1:3000!
Sleeping 15 seconds without sending any requests ...
Send request ... no response
thread 'main' panicked at examples/client.rs:33:14:
called `Result::unwrap()` on an `Err` value: Os { code: 32, kind: BrokenPipe, message: "Broken pipe" }
```

A timed probe that connects and sends nothing sees the server close the connection after 5.0 seconds.
As with the old workaround, the server closes the connection and does not answer `408 Request Timeout`.

The Torrust Tracker adopted this in [torrust/torrust-tracker#2776](https://github.com/torrust/torrust-tracker/issues/2776).
There, the same fix also closes silent TLS connections and the health check API, which the old workaround did not cover.

## Rocket

Run the server:

```output
ROCKET_KEEP_ALIVE=1 ROCKET_PORT=3000 cargo run --example rocket_server
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

## Actix Web

Run the server:

```output
cargo run --example actix_web_server
Starting server on: http://127.0.0.1:3000 ...
```

## Mitigating Slowloris DoS Attack with proxies

I'm you are using a proxy ([Nginx](https://nginx.org/en/) or [Apache](https://httpd.apache.org/)) it seems you can mitigate this attack with the proxy configuration:

- <https://www.nginx.com/blog/mitigating-ddos-attacks-with-nginx-and-nginx-plus/>
- <https://blog.imkhoi.com/posts/2023/10/slowloris-ddos-and-how-to-mitigate-with-nginx/>
- <https://hexadix.com/slowloris-dos-attack-mitigation-nginx-web-server/>
- <https://www.acunetix.com/blog/articles/slow-http-dos-attacks-mitigate-apache-http-server/>