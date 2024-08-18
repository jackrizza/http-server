extern crate httpserver;
use clap::Parser;
use httpserver::datastore::{DataStore, Op};
use httpserver::router;

/// Simple HTTP server
/// refer to https://github.com/jackrizza/http-server
/// for more information
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// change the port number. Default is 8888
    #[arg(long, default_value_t = 8888)]
    port: u16,

    /// Authenticate the user
    #[arg(short, long, default_value_t = false)]
    authenticate: bool,

    /// set password for authentication (required for authenticate flag)
    #[arg(short, long, default_value_t = String::from(""))]
    password: String,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args = Args::parse();

    let mut datastore = DataStore::new();
    datastore.listen();

    let is_password = || match args.password == "" {
        true => panic!("Password is required"),
        false => datastore.send(Op::Upsert("Password".into(), args.password)),
    };
    match args.authenticate {
        true => {
            datastore.send(Op::Upsert("Authenticate".into(), "true".into()));
            is_password();
        }
        false => (),
    }

    router(args.port, &mut datastore).await
}
