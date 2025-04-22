use crate::auth::allowed_user;
use crate::datastore::{DataStore, Op};
use actix_session::Session;
use actix_web::http::header::LOCATION;
use actix_web::{HttpRequest, HttpResponse, Responder, get, post, web};
use nanoid::nanoid;
use serde::Deserialize;
use log;

#[derive(Debug, Deserialize)]
struct Login {
    password: String,
}

#[get("/login")]
pub async fn login() -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(include_str!("../www/login.html"))
}

#[post("/login")]
pub async fn post_login(
    req: HttpRequest,
    password: web::Form<Login>,
    session: Session,
    data: web::Data<DataStore>,
) -> impl Responder {
    let mut ds = data.as_ref().clone();
    if allowed_user(password.password.clone(), &mut ds).await {
        let ip = match req.peer_addr() {
            Some(ip) => ip.ip().to_string(),
            None => "unknown".to_string(),
        };
        let session_id = nanoid!();
        let session_key = nanoid!();
        // log::info!("ip : {}, id : {}, key : {}", ip, session_id, session_key);

        ds.send(Op::Upsert(session_id.clone(), session_key.clone()));
        ds.send(Op::Upsert(ip, session_id.clone()));
        // let cookie = format!("session={}", session_id);
        match session.insert("session_key", session_key) {
            Ok(_) => {
                // session.renew();
                log::debug!("session set");
                return HttpResponse::SeeOther()
                    .insert_header((LOCATION, "/#/."))
                    .finish();
            }
            Err(_) => {
                return HttpResponse::SeeOther()
                    .insert_header((LOCATION, "/login?toast=cookienotset"))
                    .finish();
            }
        }
    } else {
        return HttpResponse::SeeOther()
            .insert_header((LOCATION, "/login?toast=wrongpassword"))
            .finish();
    }
}
