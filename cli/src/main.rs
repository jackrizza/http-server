use clap::Parser;
use http_server::datastore::{DataStore, Op};
use http_server::{http_router, https_router};

/// Browse, share, and upload files from the current directory.
#[derive(Parser, Debug)]
#[command(name = "http-server", version, about, long_about = None)]
struct Args {
    /// change the port number. Default is 8888
    #[arg(long, default_value_t = 8888)]
    port: u16,

    /// Require a password over HTTPS (certificate and key required)
    #[arg(short, long, default_value_t = false)]
    authenticate: bool,

    /// set password for authentication (required for authenticate flag)
    #[arg(short, long, required_if_eq("authenticate", "true"))]
    password: Option<String>,

    #[arg(long, default_value_t = String::from("key.pem"))]
    pem_file: String,

    #[arg(long, default_value_t = String::from("cert.pem"))]
    cert_file: String,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args = Args::parse();

    eprintln!("Sharing {}", std::env::current_dir()?.display());
    let (scheme, port) = if args.authenticate {
        ("https", 8443)
    } else {
        ("http", args.port)
    };
    eprintln!("Open {scheme}://localhost:{port}/ (Ctrl+C to stop)");

    let mut datastore = DataStore::new();
    datastore.listen();

    match args.authenticate {
        true => {
            let password = match args.password {
                Some(p) => p,
                None => {
                    println!("Password is required for authentication");
                    return Ok(());
                }
            };
            datastore.send(Op::Upsert("Authenticate".into(), "true".into()));
            datastore.send(Op::Upsert("Password".into(), password));

            https_router(&mut datastore, args.pem_file, args.cert_file).await
        }
        false => http_router(args.port, &mut datastore).await,
    }
}
