use actix_session::Session;
use actix_web::http::header::LOCATION;
use actix_web::{get, post, web, HttpResponse, Responder};

use actix_multipart::form::{tempfile::TempFile, text::Text, MultipartForm};

use crate::datastore::DataStore;

use crate::auth::auth_chain;

#[get("/upload_file")]
pub async fn get_upload_file(data: web::Data<DataStore>, session: Session) -> impl Responder {
    let mut ds = data.as_ref().clone();
    let key = match session.get::<String>("session") {
        Ok(Some(key)) => key,
        _ => "".to_string(),
    };
    println!("key : {}", key);
    if !auth_chain(key, &mut ds).await {
        return HttpResponse::SeeOther()
            .insert_header((LOCATION, "/login"))
            .finish();
    }
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(include_str!("../www/upload.html"))
}

#[derive(Debug, MultipartForm)]
struct UploadForm {
    #[multipart(limit = "100MB")]
    file: TempFile,
    path: Text<String>,
}

#[post("/upload_file")]
pub async fn post_upload_file(
    MultipartForm(form): MultipartForm<UploadForm>,
    data: web::Data<DataStore>,
    session: Session,
) -> impl Responder {
    let key = match session.get::<String>("session") {
        Ok(Some(key)) => key,
        _ => "".to_string(),
    };

    let mut ds = data.as_ref().clone();
    if !auth_chain(key, &mut ds).await {
        return HttpResponse::SeeOther()
            .insert_header((LOCATION, "/login"))
            .finish();
    }

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
