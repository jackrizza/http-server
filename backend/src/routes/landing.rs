use actix_web::{HttpResponse, Responder, get};

#[get("/")]
pub async fn landing() -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(include_str!("../www/index.html"))
}
