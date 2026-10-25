use crate::routes::api::{failure, wants_json};
use actix_web::http::header::LOCATION;
use actix_web::{HttpRequest, HttpResponse, Responder, http::StatusCode, post};
use serde_json::json;
use std::{fs, io::ErrorKind};

use actix_multipart::form::{MultipartForm, tempfile::TempFile, text::Text};

#[derive(Debug, MultipartForm)]
struct UploadForm {
    #[multipart(limit = "100MB")]
    file: TempFile,
    path: Text<String>,
}

#[post("/upload_file")]
pub async fn post_upload_file(
    req: HttpRequest,
    MultipartForm(form): MultipartForm<UploadForm>,
) -> impl Responder {
    let Some(name) = form.file.file_name else {
        return failure(&req, StatusCode::BAD_REQUEST, "A filename is required");
    };
    let path = format!("./{}/{}", form.path.to_string(), name);
    if let Err(error) = form.file.file.persist(&path) {
        // Renaming a system temporary file can fail when the shared directory
        // is on another drive. Copy it instead; dropping the tempfile cleans up
        // the original after the copy completes.
        if error.error.kind() != ErrorKind::CrossesDevices {
            return failure(&req, StatusCode::BAD_REQUEST, &error.error.to_string());
        }
        if let Err(error) = fs::copy(error.file.path(), &path) {
            return failure(&req, StatusCode::BAD_REQUEST, &error.to_string());
        }
    }

    if wants_json(&req) {
        return HttpResponse::Ok()
            .json(json!({"ok": true, "path": form.path.to_string(), "name": name}));
    }

    HttpResponse::SeeOther()
        .insert_header((LOCATION, format!("/#/{}", form.path.to_string())))
        .finish()
}
