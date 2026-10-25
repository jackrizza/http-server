use std::{
    io::{Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    process::{Child, Command, Stdio},
    thread,
    time::{Duration, Instant},
};

struct Server(Child);

impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn get(address: SocketAddr, path: &str) -> String {
    let mut stream = TcpStream::connect_timeout(&address, Duration::from_secs(2)).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();
    write!(
        stream,
        "GET {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n"
    )
    .unwrap();
    let mut response = String::new();
    stream.read_to_string(&mut response).unwrap();
    assert!(response.starts_with("HTTP/1.1 200"), "{path}: {response}");
    response
}

#[test]
fn command_reports_its_installed_name_and_version() {
    let version = Command::new(env!("CARGO_BIN_EXE_http-server"))
        .arg("--version")
        .output()
        .unwrap();
    assert!(version.status.success());
    assert_eq!(
        String::from_utf8(version.stdout).unwrap().trim(),
        concat!("http-server ", env!("CARGO_PKG_VERSION"))
    );
    let help = Command::new(env!("CARGO_BIN_EXE_http-server"))
        .arg("--help")
        .output()
        .unwrap();
    assert!(help.status.success());
    assert!(
        String::from_utf8(help.stdout)
            .unwrap()
            .contains("Usage: http-server")
    );
}

#[test]
fn serves_the_launch_directory_and_embedded_ui_outside_the_checkout() {
    // A separate process keeps the test runner's working directory unchanged.
    let directory = tempfile::tempdir().unwrap();
    std::fs::write(directory.path().join("shared file.txt"), "shared contents").unwrap();
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    drop(listener);

    let mut server = Server(
        Command::new(env!("CARGO_BIN_EXE_http-server"))
            .args(["--port", &address.port().to_string()])
            .current_dir(directory.path())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap(),
    );
    let deadline = Instant::now() + Duration::from_secs(20);
    loop {
        assert!(
            server.0.try_wait().unwrap().is_none(),
            "http-server exited before accepting requests"
        );
        if TcpStream::connect_timeout(&address, Duration::from_millis(100)).is_ok() {
            break;
        }
        assert!(Instant::now() < deadline, "http-server did not start");
        thread::sleep(Duration::from_millis(50));
    }

    let listing = get(address, "/files/");
    assert!(listing.contains("shared file.txt"));
    assert!(!listing.contains("Cargo.toml"));
    assert!(get(address, "/get/file/shared%20file.txt").ends_with("shared contents"));
    assert!(get(address, "/").contains("/cdn/spine/app.js"));
    assert!(get(address, "/cdn/spine/app.js").contains("HttpFilesApi"));
    assert!(get(address, "/cdn/spine/vendor/spine.min.js").contains("Spine"));
    assert!(get(address, "/cdn/spine/files.css").contains("{"));
}
