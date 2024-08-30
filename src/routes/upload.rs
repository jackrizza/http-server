use actix_multipart::Multipart;
use actix_session::Session;
use actix_web::http::header::LOCATION;
use actix_web::{get, post, web, HttpResponse, Responder};
use futures::StreamExt;
use std::io::Write;

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

#[post("/upload_file")]
pub async fn post_upload_file(
    mut payload: Multipart,
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
    while let Some(field) = payload.next().await {
        let (mut field, file_name) = match field {
            Ok(f) => {
                let content_type = f.content_disposition().unwrap();
                let file_name = content_type.get_filename().unwrap_or("untitled_file");
                let file_name = file_name.to_owned();
                (f, file_name)
            }
            Err(_) => {
                return HttpResponse::SeeOther()
                    .insert_header((LOCATION, "/#/"))
                    .finish()
            }
        };

        // File::create is blocking operation, use threadpool
        let mut f = match web::block(|| std::fs::File::create(file_name))
            .await
            .unwrap()
        {
            Ok(f) => f,
            Err(_) => {
                return HttpResponse::SeeOther()
                    .insert_header((LOCATION, "/#/"))
                    .finish()
            }
        };

        while let Some(chunk) = field.next().await {
            let chunk = match chunk {
                Ok(chunk) => chunk,
                Err(_) => {
                    return HttpResponse::SeeOther()
                        .insert_header((LOCATION, "/#/"))
                        .finish()
                }
            };
            let _ = f.write_all(&chunk);
        }
    }

    HttpResponse::SeeOther()
        .insert_header((LOCATION, "/#/"))
        .finish()
}
