#[macro_use]
extern crate rocket;

#[get("/")]
fn hello() -> &'static str {
    "Hello, world!"
}

#[rocket::main]
async fn main() {
    let _unused = rocket::build().mount("/", routes![hello]).launch().await;
}
