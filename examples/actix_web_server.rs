use std::time::Duration;

use actix_web::{web, App, HttpResponse, HttpServer, Responder};

async fn hello() -> impl Responder {
    println!("New request ...");
    HttpResponse::Ok().body("Hello world!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Starting server on: http://127.0.0.1:3000 ..."); // DevSkim: ignore DS137138

    HttpServer::new(|| App::new().route("/", web::get().to(hello)))
        .bind(("127.0.0.1", 3000))?
        .client_request_timeout(Duration::from_secs(5))
        .run()
        .await
}
