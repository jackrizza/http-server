use crate::auth::allowed_user;
use crate::auth_chain;
use crate::datastore::{DataStore, Op};
use actix_session::Session;
use actix_web::http::header::LOCATION;
use actix_web::{get, post, web, HttpResponse, Responder};
use nanoid::nanoid;
use serde::Deserialize;

#[get("/")]
pub async fn landing(data: web::Data<DataStore>, session: Session) -> impl Responder {
    let mut ds = data.as_ref().clone();
    println!("{:#?}", ds.all());
    println!("{:#?}", session.entries());
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
        .body(include_str!("../www/index.html"))
}

#[get("/login")]
pub async fn login() -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(include_str!("../www/login.html"))
}

#[derive(Debug, Deserialize)]
struct Login {
    password: String,
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
                    .insert_header((LOCATION, "/"))
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
