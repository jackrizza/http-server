use actix_web::{get, HttpResponse, Responder};

#[get("/")]
pub async fn landing() -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(include_str!("../www/index.html"))
}

#[get("/login")]
pub async fn login() -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(include_str!("../www/login.html"))
}
