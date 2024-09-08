use clap::Parser;
use backend::datastore::{DataStore, Op};
use backend::{http_router, https_router};

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
    /// openssl
    /// needed for authentication (Cookies won't work without it)
    #[arg(short, long, default_value_t = false)]
    authenticate: bool,

    /// set password for authentication (required for authenticate flag)
    #[arg(short, long, default_value_t = String::from(""))]
    password: String,

    #[arg(long, default_value_t = String::from("key.pem"))]
    pem_file: String,

    #[arg(long, default_value_t = String::from("cert.pem"))]
    cert_file: String,
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

            https_router(&mut datastore, args.pem_file, args.cert_file).await
        }
        false => http_router(args.port, &mut datastore).await,
    }
}
