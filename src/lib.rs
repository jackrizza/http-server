use actix_files as fs;
use actix_session::{storage::CookieSessionStore, SessionMiddleware};
use actix_web::cookie::{Key, SameSite};
use actix_web::middleware::Logger;
use actix_web::web::Redirect;
use actix_web::{dev::Service as _, web, App};
use actix_web::{guard, HttpRequest, HttpResponse, HttpServer, Responder, Result, Scope};
use datastore::{DataStore, Op};
use futures_util::future::FutureExt;

use actix_session::config::{BrowserSession, CookieContentSecurity};
use actix_web;

use routes::landing::{landing, login, post_login};
use routes::upload::{get_upload_file, post_upload_file};
use routes::{normalize_css, skeleton_css};

pub mod auth;
pub mod datastore;
pub mod routes;

fn session_middleware() -> SessionMiddleware<CookieSessionStore> {
    SessionMiddleware::builder(CookieSessionStore::default(), Key::generate())
        .cookie_name(String::from("simplehttpkey")) // arbitrary name
        // .cookie_secure(true) // https only
        // .session_lifecycle(BrowserSession::default()) // expire at end of session
        // .cookie_same_site(SameSite::Strict)
        // .cookie_content_security(CookieContentSecurity::Private) // encrypt
        // .cookie_http_only(true) // disallow scripts from reading
        .build()
}

pub async fn auth_chain(key: String, ds: &mut DataStore) -> bool {
    if auth::is_auth_required(ds).await {
        if auth::allowed_session(key, ds).await {
            return true;
        } else {
            return false;
        }
    } else {
        return true;
    }
}

pub async fn router(port: u16, ds: &mut DataStore) -> std::io::Result<()> {
    let mut ds = web::Data::new(ds.clone());

    // configuration file or environment variables.
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    HttpServer::new(move || {
        App::new()
            .app_data(ds.clone())
            .wrap(session_middleware())
            .wrap(Logger::default())
            .service(fs::Files::new("/static", ".").show_files_listing())
            .service(normalize_css)
            .service(skeleton_css)
            .service(get_upload_file)
            // .route("/upload_file", web::get().to( || get_upload_file(&mut ds)))
            .service(post_upload_file)
            .service(landing)
            .service(login)
            .service(post_login)
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}
