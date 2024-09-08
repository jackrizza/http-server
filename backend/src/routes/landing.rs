use crate::auth::auth_chain;
use crate::datastore::DataStore;
use actix_session::Session;
use actix_web::http::header::LOCATION;
use actix_web::{get, web, HttpResponse, Responder};

#[get("/")]
pub async fn landing(data: web::Data<DataStore>, session: Session) -> impl Responder {
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
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(include_str!("../www/index.html"))
}
