use crate::auth::allowed_user;
use crate::datastore::{DataStore, Op};
use actix_session::Session;
use actix_web::http::header::LOCATION;
use actix_web::{get, post, web, HttpResponse, Responder};
use nanoid::nanoid;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Login {
    password: String,
}

#[get("/login")]
pub async fn login() -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(include_str!("../www2/login.html"))
}

#[post("/login")]
pub async fn post_login(
    password: web::Form<Login>,
    session: Session,
    data: web::Data<DataStore>,
) -> impl Responder {
    let mut ds = data.as_ref().clone();
    if allowed_user(password.password.clone(), &mut ds).await {
        let session_id = nanoid!();
        ds.send(Op::Upsert("session".into(), session_id.clone()));
        // let cookie = format!("session={}", session_id);
        match session.insert("session", session_id) {
            Ok(_) => {
                return HttpResponse::SeeOther()
                    .insert_header((LOCATION, "/#/"))
                    .finish()
            }
            Err(_) => {
                return HttpResponse::SeeOther()
                    .insert_header((LOCATION, "/login?toast=cookienotset"))
                    .finish()
            }
        }
    } else {
        return HttpResponse::SeeOther()
            .insert_header((LOCATION, "/login?toast=wrongpassword"))
            .finish();
    }
}
