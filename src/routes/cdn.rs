use actix_web::{get, HttpResponse, Responder};
use bytes::Bytes;

#[get("/cdn/css/app.css")]
pub async fn css_app() -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/css; charset=utf-8")
        .body(include_str!("../www2/css/app.css"))
}

#[get("/cdn/css/foundation.css")]
pub async fn css_foundation() -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/css; charset=utf-8")
        .body(include_str!("../www2/css/foundation.css"))
}

#[get("/cdn/js/app.js")]
pub async fn js_app() -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/css; charset=utf-8")
        .body(include_str!("../www2/js/app.js"))
}

#[get("/cdn/js/login.js")]
pub async fn js_login() -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/css; charset=utf-8")
        .body(include_str!("../www2/js/login.js"))
}

#[get("/cdn/js/navagation.js")]
pub async fn js_navagation() -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/css; charset=utf-8")
        .body(include_str!("../www2/js/navagation.js"))
}

#[get("/cdn/js/show_file.js")]
pub async fn js_show_file() -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/css; charset=utf-8")
        .body(include_str!("../www2/js/show_file.js"))
}

#[get("/cdn/js/table_builder.js")]
pub async fn js_table_builder() -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/css; charset=utf-8")
        .body(include_str!("../www2/js/table_builder.js"))
}

#[get("/cdn/js/upload.js")]
pub async fn js_upload() -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/css; charset=utf-8")
        .body(include_str!("../www2/js/upload.js"))
}

#[get("/favicon.ico")]
pub async fn favicon() -> impl Responder {
    HttpResponse::Ok().body(Bytes::from_static(include_bytes!(
        "../www2/icons/favicon.ico"
    )))
}

#[get("cdn/icons/cpp.png")]
pub async fn cpp() -> impl Responder {
    HttpResponse::Ok().body(Bytes::from_static(include_bytes!("../www2/icons/cpp.png")))
}

#[get("cdn/icons/cs.png")]
pub async fn cs() -> impl Responder {
    HttpResponse::Ok().body(Bytes::from_static(include_bytes!("../www2/icons/cs.png")))
}

#[get("cdn/icons/css.png")]
pub async fn css() -> impl Responder {
    HttpResponse::Ok().body(Bytes::from_static(include_bytes!("../www2/icons/css.png")))
}

#[get("cdn/icons/csv.png")]
pub async fn csv() -> impl Responder {
    HttpResponse::Ok().body(Bytes::from_static(include_bytes!("../www2/icons/csv.png")))
}

#[get("cdn/icons/dwg.png")]
pub async fn dwg() -> impl Responder {
    HttpResponse::Ok().body(Bytes::from_static(include_bytes!("../www2/icons/dwg.png")))
}

#[get("cdn/icons/file.png")]
pub async fn file() -> impl Responder {
    HttpResponse::Ok().body(Bytes::from_static(include_bytes!("../www2/icons/file.png")))
}

#[get("cdn/icons/folder.png")]
pub async fn folder() -> impl Responder {
    HttpResponse::Ok().body(Bytes::from_static(include_bytes!(
        "../www2/icons/folder.png"
    )))
}

#[get("cdn/icons/html.png")]
pub async fn html() -> impl Responder {
    HttpResponse::Ok().body(Bytes::from_static(include_bytes!("../www2/icons/html.png")))
}

#[get("cdn/icons/img.png")]
pub async fn img() -> impl Responder {
    HttpResponse::Ok().body(Bytes::from_static(include_bytes!("../www2/icons/img.png")))
}

#[get("cdn/icons/jpg.png")]
pub async fn jpg() -> impl Responder {
    HttpResponse::Ok().body(Bytes::from_static(include_bytes!("../www2/icons/jpg.png")))
}

#[get("cdn/icons/js.png")]
pub async fn js() -> impl Responder {
    HttpResponse::Ok().body(Bytes::from_static(include_bytes!("../www2/icons/js.png")))
}

#[get("cdn/icons/json.png")]
pub async fn json() -> impl Responder {
    HttpResponse::Ok().body(Bytes::from_static(include_bytes!("../www2/icons/json.png")))
}

#[get("cdn/icons/pdf.png")]
pub async fn pdf() -> impl Responder {
    HttpResponse::Ok().body(Bytes::from_static(include_bytes!("../www2/icons/pdf.png")))
}

#[get("cdn/icons/php.png")]
pub async fn php() -> impl Responder {
    HttpResponse::Ok().body(Bytes::from_static(include_bytes!("../www2/icons/php.png")))
}

#[get("cdn/icons/png.png")]
pub async fn png() -> impl Responder {
    HttpResponse::Ok().body(Bytes::from_static(include_bytes!("../www2/icons/png.png")))
}

#[get("cdn/icons/sql.png")]
pub async fn sql() -> impl Responder {
    HttpResponse::Ok().body(Bytes::from_static(include_bytes!("../www2/icons/sql.png")))
}

#[get("cdn/icons/txt.png")]
pub async fn txt() -> impl Responder {
    HttpResponse::Ok().body(Bytes::from_static(include_bytes!("../www2/icons/txt.png")))
}

#[get("cdn/icons/video.png")]
pub async fn video() -> impl Responder {
    HttpResponse::Ok().body(Bytes::from_static(include_bytes!(
        "../www2/icons/video.png"
    )))
}

#[get("cdn/icons/word.png")]
pub async fn word() -> impl Responder {
    HttpResponse::Ok().body(Bytes::from_static(include_bytes!("../www2/icons/word.png")))
}

#[get("cdn/icons/xls.png")]
pub async fn xls() -> impl Responder {
    HttpResponse::Ok().body(Bytes::from_static(include_bytes!("../www2/icons/xls.png")))
}
