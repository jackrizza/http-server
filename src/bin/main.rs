extern crate httpserver;

use actix_files as fs;
use actix_session::{storage::CookieSessionStore, SessionMiddleware};
use actix_web::cookie::Key;
use actix_web::middleware::Logger;
use actix_web::{App, HttpServer};

use httpserver::routes::landing::landing;
use httpserver::routes::upload::{get_upload_file, post_upload_file};
use httpserver::routes::{normalize_css, skeleton_css};

fn secrect_key() -> Key {
    Key::generate()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // configuration file or environment variables.

    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    HttpServer::new(|| {
        App::new()
            .wrap(SessionMiddleware::new(
                CookieSessionStore::default(),
                secrect_key().clone(),
            ))
            .wrap(Logger::default())
            .service(fs::Files::new("/static", ".").show_files_listing())
            .service(normalize_css)
            .service(skeleton_css)
            .service(get_upload_file)
            .service(post_upload_file)
            .service(landing)
    })
    .bind(("127.0.0.1", 8888))?
    .run()
    .await
}
