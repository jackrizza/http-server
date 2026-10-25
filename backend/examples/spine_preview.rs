// Local development host using the same embedded assets and HTTP handlers as
// the main CLI. A separate executable avoids replacing a running CLI binary.
use actix_web::{App, HttpServer, web};
use http_server::datastore::DataStore;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let port = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "8889".into())
        .parse::<u16>()
        .expect("port must be an integer");
    let mut store = DataStore::new();
    store.listen();
    println!("Spine preview: http://127.0.0.1:{port}/#/files/");
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(store.clone()))
            .configure(http_server::configure_public)
            .configure(http_server::configure_protected_and_static)
    })
    .bind(("127.0.0.1", port))?
    .run()
    .await
}
