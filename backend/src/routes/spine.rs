use actix_web::{HttpResponse, Responder, get, web};

#[get("/spine")]
pub async fn page() -> impl Responder {
    HttpResponse::PermanentRedirect()
        .insert_header(("Location", "/"))
        .finish()
}

// Consumer-owned UI files and a frozen third-party runtime are embedded by Rust.
#[get("/cdn/spine/{path:.*}")]
pub async fn asset(path: web::Path<String>) -> HttpResponse {
    let (mime, content): (&str, &'static [u8]) = match path.as_str() {
        "app.js" => (
            "application/javascript",
            include_bytes!("../www/spine-ui/app.js"),
        ),
        "login.js" => (
            "application/javascript",
            include_bytes!("../www/spine-ui/login.js"),
        ),
        "files-api.js" => (
            "application/javascript",
            include_bytes!("../www/spine-ui/files-api.js"),
        ),
        "files.js" => (
            "application/javascript",
            include_bytes!("../www/spine-ui/files.js"),
        ),
        "files.css" => ("text/css", include_bytes!("../www/spine-ui/files.css")),
        "vendor/spine.js" => (
            "application/javascript",
            include_bytes!("../www/vendor/spine/spine.js"),
        ),
        "vendor/spine.min.js" => (
            "application/javascript",
            include_bytes!("../www/vendor/spine/spine.min.js"),
        ),
        "vendor/spine.min.js.map" => (
            "application/json",
            include_bytes!("../www/vendor/spine/spine.min.js.map"),
        ),
        "vendor/manifest.json" => (
            "application/json",
            include_bytes!("../www/vendor/spine/manifest.json"),
        ),
        _ => return HttpResponse::NotFound().finish(),
    };
    HttpResponse::Ok().content_type(mime).body(content)
}
