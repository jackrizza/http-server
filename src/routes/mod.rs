pub mod landing;
pub mod upload;

use actix_web::{get, HttpResponse, Responder};

#[get("/css/normalize.css")]
pub async fn normalize_css() -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/css; charset=utf-8")
        .body(include_str!("../www/normalize.css"))
}

#[get("/css/skeleton.css")]
pub async fn skeleton_css() -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/css; charset=utf-8")
        .body(include_str!("../www/skeleton.css"))
}
