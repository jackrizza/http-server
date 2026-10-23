use actix_web::{HttpResponse, Responder, get};

#[get("/cdn/css/foundation.css")]
pub async fn css_foundation() -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/css; charset=utf-8")
        .body(include_str!("../www/css/foundation.css"))
}

#[get("/favicon.ico")]
pub async fn favicon() -> impl Responder {
    HttpResponse::Ok()
        .content_type("image/x-icon")
        .body(include_bytes!("../www/icons/favicon.ico").as_slice())
}
