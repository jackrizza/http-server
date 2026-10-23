use actix_web::{HttpRequest, HttpResponse, http::StatusCode};
use serde_json::json;

// Opt in so existing forms keep their redirect-based behavior.
pub fn wants_json(req: &HttpRequest) -> bool {
    req.headers()
        .get("accept")
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| {
            value
                .split(',')
                .any(|part| part.trim().starts_with("application/json"))
        })
}

pub fn failure(req: &HttpRequest, status: StatusCode, message: &str) -> HttpResponse {
    if wants_json(req) {
        HttpResponse::build(status).json(json!({ "error": message }))
    } else {
        HttpResponse::build(status)
            .content_type("text/plain; charset=utf-8")
            .body(message.to_owned())
    }
}
