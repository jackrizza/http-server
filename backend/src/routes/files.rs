use crate::routes::api::{failure, wants_json};
use actix_files::NamedFile;
use actix_multipart::form::{MultipartForm, text::Text};
use actix_web::http::StatusCode;
use actix_web::http::header::LOCATION;
use actix_web::{HttpRequest, HttpResponse, Responder, Result, get, post, web};
use chrono::DateTime;
use chrono::offset::Utc;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs;
use std::path::PathBuf;

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
    size: Option<u64>,
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
    req: HttpRequest,
    MultipartForm(form): MultipartForm<NewFolder>,
) -> impl Responder {
    println!("new folder : {:#?}", form);

    let path = format!("./{}/{}", form.path.to_string(), form.name.to_string());
    if let Err(error) = fs::create_dir_all(path) {
        return failure(&req, StatusCode::BAD_REQUEST, &error.to_string());
    }
    if wants_json(&req) {
        return HttpResponse::Ok().json(
            json!({"ok": true, "path": form.path.to_string(), "name": form.name.to_string()}),
        );
    }

    return HttpResponse::SeeOther()
        .insert_header((LOCATION, format!("/#/{}", form.path.to_string())))
        .finish();
}

#[get("/get/file/{tail:.*}")]
pub async fn get_file(req: HttpRequest) -> Result<NamedFile> {
    let tail: String = req.match_info().get("tail").unwrap().parse().unwrap();

    Ok(NamedFile::open(tail)?)
}

#[get("/get/video/{filename:.*}")]
async fn get_video(path: web::Path<String>) -> Result<NamedFile> {
    let filename = path.into_inner();
    let full_path: PathBuf = PathBuf::from(".").join(&filename);

    // NamedFile implements Responder, handles Range → 206, sniffing MIME, sendfile, etc.
    Ok(NamedFile::open(full_path)?
        .use_last_modified(true)
        .set_content_disposition(actix_web::http::header::ContentDisposition {
            disposition: actix_web::http::header::DispositionType::Inline,
            parameters: vec![actix_web::http::header::DispositionParam::Filename(
                filename,
            )],
        }))
}

#[get("/files/{tail:.*}")]
pub async fn all(req: HttpRequest) -> impl Responder {
    let mut files = Files::new();

    let tail: String = req
        .match_info()
        .get("tail")
        .unwrap_or("".into())
        .parse()
        .unwrap_or("".into());

    println!("tail: {:#?}", tail);

    let entries = match fs::read_dir(format!("./{}", tail)) {
        Ok(entries) => entries,
        Err(error) => return failure(&req, StatusCode::NOT_FOUND, &error.to_string()),
    };
    for path in entries {
        let entry = match path {
            Ok(entry) => entry,
            Err(_) => continue,
        };
        let p = entry.path().display().to_string().replace('\\', "/");
        let name = entry.file_name().to_string_lossy().to_string();
        let metadata = match entry.metadata() {
            Ok(metadata) => metadata,
            Err(_) => continue,
        };
        let is_file = metadata.is_file();
        let size = is_file.then_some(metadata.len());
        let created = metadata
            .created()
            .map(|created| DateTime::<Utc>::from(created).to_rfc2822())
            .unwrap_or_default();

        if is_file {
            files.0.push(About::File(Single {
                name,
                created,
                path: p,
                size,
            }));
        } else {
            files.0.push(About::Folder(Single {
                name,
                created,
                path: p,
                size,
            }));
        }
    }

    HttpResponse::Ok().json(files)
}
