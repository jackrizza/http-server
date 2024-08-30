use actix_files as fs;
use actix_session::{storage::CookieSessionStore, SessionMiddleware};
use actix_web::cookie::{Key, SameSite};
use actix_web::middleware::Logger;
use actix_web::{get, web, App, HttpResponse, Responder};
use actix_web::{HttpServer, Result};
use datastore::DataStore;

use actix_session::config::{BrowserSession, CookieContentSecurity};
use actix_web;

use routes::files::{all, get_file};
use routes::landing::landing;
use routes::login::{login, post_login};
use routes::upload::{get_upload_file, post_upload_file};
use routes::{normalize_css, skeleton_css};

pub mod auth;
pub mod datastore;
pub mod routes;

use std::{fs::File, io::BufReader};

fn session_middleware() -> SessionMiddleware<CookieSessionStore> {
    SessionMiddleware::builder(CookieSessionStore::default(), Key::generate())
        .cookie_name(String::from("simplehttpkey")) // arbitrary name
        .cookie_secure(true) // https only
        .session_lifecycle(BrowserSession::default()) // expire at end of session
        .cookie_same_site(SameSite::Strict)
        .cookie_content_security(CookieContentSecurity::Private) // encrypt
        .cookie_http_only(true) // disallow scripts from reading
        .build()
}

#[get("/favicon.ico")]
pub async fn favicon() -> impl Responder {
    let image_content = match web::block(|| std::fs::read("src/www2/icons/favicon.ico")).await {
        Ok(image) => image.unwrap(),
        _ => Vec::new(),
    };
    HttpResponse::Ok()
        .content_type("image/jpeg")
        .body(image_content)
}

pub async fn http_router(port: u16, ds: &mut DataStore) -> std::io::Result<()> {
    let ds = web::Data::new(ds.clone());

    // configuration file or environment variables.
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    HttpServer::new(move || {
        App::new()
            .app_data(ds.clone())
            .wrap(session_middleware())
            .wrap(Logger::default())
            .service(fs::Files::new("/api", "./src/www2").show_files_listing())
            .service(fs::Files::new("/static", ".").show_files_listing())
            .service(normalize_css)
            .service(skeleton_css)
            .service(get_upload_file)
            // .route("/upload_file", web::get().to( || get_upload_file(&mut ds)))
            .service(post_upload_file)
            .service(landing)
            .service(login)
            .service(post_login)
            .service(favicon)
            .service(all)
            .service(get_file)
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}

pub async fn https_router(
    ds: &mut DataStore,
    pem_file: String,
    cert_file: String,
) -> std::io::Result<()> {
    let ds = web::Data::new(ds.clone());
    // build TLS config from files
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .unwrap();

    let mut certs_file = BufReader::new(File::open(&cert_file).unwrap());
    let mut key_file = BufReader::new(File::open(&pem_file).unwrap());

    // load TLS certs and key
    // to create a self-signed temporary cert for testing:
    // `openssl req -x509 -newkey rsa:4096 -nodes -keyout key.pem -out cert.pem -days 365 -subj '/CN=localhost'`
    let tls_certs = rustls_pemfile::certs(&mut certs_file)
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    let tls_key = rustls_pemfile::pkcs8_private_keys(&mut key_file)
        .next()
        .unwrap()
        .unwrap();

    // set up TLS config options
    let tls_config = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(tls_certs, rustls::pki_types::PrivateKeyDer::Pkcs8(tls_key))
        .unwrap();

    // configuration file or environment variables.
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    // log::info!("starting HTTPS server at http://0.0.0.0:8443");
    HttpServer::new(move || {
        App::new()
            .app_data(ds.clone())
            .wrap(session_middleware())
            .wrap(Logger::default())
            .service(fs::Files::new("/api", "./src/www2").show_files_listing())
            .service(fs::Files::new("/static", ".").show_files_listing())
            .service(normalize_css)
            .service(skeleton_css)
            .service(get_upload_file)
            // .route("/upload_file", web::get().to( || get_upload_file(&mut ds)))
            .service(post_upload_file)
            .service(landing)
            .service(login)
            .service(post_login)
            .service(favicon)
            .service(all)
            .service(get_file)
    })
    .bind_rustls_0_23(("0.0.0.0", 8443), tls_config)?
    .workers(2)
    .run()
    .await
}
