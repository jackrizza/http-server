use actix_multipart::Multipart;
use actix_web::web::Redirect;
use actix_web::{get, post, web, HttpResponse, Responder};
use futures::StreamExt;
use std::io::Write;

#[get("/upload_file")]
pub async fn get_upload_file() -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(include_str!("../www/upload.html"))
}

#[post("/upload_file")]
pub async fn post_upload_file(mut payload: Multipart) -> impl Responder {
    while let Some(field) = payload.next().await {
        let (mut field, file_name) = match field {
            Ok(f) => {
                let content_type = f.content_disposition().unwrap();
                let file_name = content_type.get_filename().unwrap_or("untitled_file");
                let file_name = file_name.to_owned();
                (f, file_name)
            }
            Err(e) => return Redirect::to("/upload_file?toast=error"),
        };

        // File::create is blocking operation, use threadpool
        let mut f = match web::block(|| std::fs::File::create(file_name))
            .await
            .unwrap()
        {
            Ok(f) => f,
            Err(e) => return Redirect::to("/upload_file?toast=error"),
        };

        while let Some(chunk) = field.next().await {
            let chunk = match chunk {
                Ok(chunk) => chunk,
                Err(e) => return Redirect::to("/upload_file?toast=error"),
            };
            let _ = f.write_all(&chunk);
        }
    }

    Redirect::to("/upload_file?toast=success")
}
