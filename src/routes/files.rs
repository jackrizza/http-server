use crate::auth::auth_chain;
use crate::datastore::DataStore;
use actix_files::NamedFile;
use actix_multipart::form::{text::Text, MultipartForm};
use actix_session::Session;
use actix_web::http::header::LOCATION;
use actix_web::{get, post, web, HttpRequest, HttpResponse, Responder, Result};
use chrono::offset::Utc;
use chrono::DateTime;
use serde::{Deserialize, Serialize};
use std::fs;
use walkdir::WalkDir;
#[derive(Debug, Serialize, Deserialize)]
enum About {
    File(Single),
    Folder(Single),
}

#[derive(Debug, Serialize, Deserialize)]
struct Single {
    name: String,
    created: String,
    path: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Files(Vec<About>);

impl Files {
    pub fn new() -> Self {
        Files { 0: Vec::new() }
    }
}

#[derive(Debug, MultipartForm)]
struct NewFolder {
    path: Text<String>,
    name: Text<String>,
}

#[post("/api/new_folder")]
pub async fn new_folder(
    MultipartForm(form): MultipartForm<NewFolder>,
    data: web::Data<DataStore>,
    session: Session,
) -> impl Responder {
    println!("new folder : {:#?}", form);
    let mut ds = data.as_ref().clone();
    let key = match session.get::<String>("session") {
        Ok(Some(key)) => key,
        _ => "".to_string(),
    };
    if !auth_chain(key, &mut ds).await {
        return HttpResponse::SeeOther()
            .insert_header((LOCATION, "/login"))
            .finish();
    }

    let path = format!("./{}/{}", form.path.to_string(), form.name.to_string());
    fs::create_dir_all(path).unwrap();

    return HttpResponse::SeeOther()
        .insert_header((LOCATION, format!("/#/{}", form.path.to_string())))
        .finish();
}

#[get("/get/file/{tail:.*}")]
pub async fn get_file(
    req: HttpRequest,
    data: web::Data<DataStore>,
    session: Session,
) -> Result<NamedFile> {
    let mut ds = data.as_ref().clone();
    let key = match session.get::<String>("session") {
        Ok(Some(key)) => key,
        _ => "".to_string(),
    };
    if !auth_chain(key, &mut ds).await {
        // return Err(ErrorUnauthorized("Access Denied"));
    }

    let tail: String = req.match_info().get("tail").unwrap().parse().unwrap();

    Ok(NamedFile::open(tail)?)
}

#[get("/files")]
pub async fn all(data: web::Data<DataStore>, session: Session) -> impl Responder {
    let mut ds = data.as_ref().clone();
    let key = match session.get::<String>("session") {
        Ok(Some(key)) => key,
        _ => "".to_string(),
    };
    if !auth_chain(key, &mut ds).await {
        // return HttpResponse::SeeOther()
        //     .insert_header((LOCATION, "/login"))
        //     .finish();
    }

    let mut files = Files::new();

    for entry in WalkDir::new(".") {
        let entry = entry.unwrap();
        let file_type = entry.file_type();

        if file_type.is_dir() {
            files.0.push(About::Folder(Single {
                name: entry.file_name().to_str().unwrap().to_string(),
                created: String::new(),
                path: entry.path().display().to_string(),
            }));
        }

        if file_type.is_file() {
            let metadata = fs::metadata(entry.path().display().to_string());
            let created = match metadata {
                Ok(meta) => {
                    let system_time = meta.created().unwrap();
                    let datetime: DateTime<Utc> = system_time.into();
                    format!("{}", datetime.format("%d/%m/%Y %T"))
                }
                _ => String::new(),
            };
            files.0.push(About::File(Single {
                name: entry.file_name().to_str().unwrap().to_string(),
                created,
                path: entry.path().display().to_string(),
            }));
        }
    }

    web::Json(files)
}
