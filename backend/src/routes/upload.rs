use actix_web::http::header::LOCATION;
use actix_web::{post, HttpResponse, Responder};

use actix_multipart::form::{tempfile::TempFile, text::Text, MultipartForm};

#[derive(Debug, MultipartForm)]
struct UploadForm {
    #[multipart(limit = "100MB")]
    file: TempFile,
    path: Text<String>,
}

#[post("/upload_file")]
pub async fn post_upload_file(MultipartForm(form): MultipartForm<UploadForm>) -> impl Responder {
    let path = format!(
        "./{}/{}",
        form.path.to_string(),
        form.file.file_name.unwrap()
    );
    form.file.file.persist(path).unwrap();

    HttpResponse::SeeOther()
        .insert_header((LOCATION, format!("/#/{}", form.path.to_string())))
        .finish()
}
