use actix_session::{SessionMiddleware, storage::CookieSessionStore};
use actix_web::{
    App,
    cookie::Key,
    http::{StatusCode, header},
    test, web,
};
use http_server::{
    auth::AuthMiddleware,
    datastore::{DataStore, Op},
    routes::{files, upload},
};
use serde_json::Value;
use std::{
    fs,
    net::SocketAddr,
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};

static FIXTURE_NUMBER: AtomicUsize = AtomicUsize::new(0);

struct Fixture {
    root: PathBuf,
    name: String,
}
impl Fixture {
    fn new() -> Self {
        let parent = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        assert_eq!(
            fs::canonicalize(std::env::current_dir().unwrap()).unwrap(),
            fs::canonicalize(&parent).unwrap()
        );
        let name = format!(
            "spine-fixture-{}-{}",
            std::process::id(),
            FIXTURE_NUMBER.fetch_add(1, Ordering::Relaxed)
        );
        let root = parent.join(&name);
        fs::create_dir(&root).unwrap();
        fs::create_dir(root.join("reports")).unwrap();
        fs::write(root.join("hello world.txt"), b"preview content").unwrap();
        fs::write(root.join("picture.png"), b"\x89PNG\r\n\x1a\n").unwrap();
        fs::write(root.join("document.pdf"), b"%PDF-1.7 fixture").unwrap();
        fs::write(root.join("clip.mp4"), b"0123456789").unwrap();
        Self { root, name }
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let parent = fs::canonicalize(env!("CARGO_MANIFEST_DIR")).unwrap();
        let root = fs::canonicalize(&self.root).unwrap();
        assert_eq!(root.parent(), Some(parent.as_path()));
        assert!(self.name.starts_with("spine-fixture-"));
        fs::remove_dir_all(root).unwrap();
    }
}

fn multipart(fields: &[(&str, &str)], file: Option<(&str, &str)>) -> String {
    let mut body = String::new();
    for (name, value) in fields {
        body.push_str(&format!("--spine-test-boundary\r\nContent-Disposition: form-data; name=\"{name}\"\r\n\r\n{value}\r\n"));
    }
    if let Some((name, value)) = file {
        body.push_str(&format!("--spine-test-boundary\r\nContent-Disposition: form-data; name=\"file\"; filename=\"{name}\"\r\nContent-Type: text/plain\r\n\r\n{value}\r\n"));
    }
    body.push_str("--spine-test-boundary--\r\n");
    body
}

#[actix_web::test]
async fn listing_previews_downloads_and_opt_in_mutations_preserve_native_forms() {
    let fixture = Fixture::new();
    let app = test::init_service(
        App::new()
            .service(files::all)
            .service(files::get_file)
            .service(files::get_video)
            .service(files::new_folder)
            .service(upload::post_upload_file),
    )
    .await;
    let response = test::call_service(
        &app,
        test::TestRequest::get()
            .uri(&format!("/files/{}", fixture.name))
            .insert_header((header::ACCEPT, "application/json"))
            .to_request(),
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    let entries: Value = test::read_body_json(response).await;
    let entries = entries.as_array().unwrap();
    assert!(entries.iter().any(|entry| {
        entry
            .get("Folder")
            .is_some_and(|folder| folder["name"] == "reports")
    }));
    assert!(entries.iter().any(|entry| {
        entry
            .get("File")
            .is_some_and(|file| file["name"] == "hello world.txt")
    }));
    let text_file = entries
        .iter()
        .find_map(|entry| {
            entry
                .get("File")
                .filter(|file| file["name"] == "hello world.txt")
        })
        .unwrap();
    assert_eq!(text_file["size"], b"preview content".len());
    let folder = entries
        .iter()
        .find_map(|entry| entry.get("Folder"))
        .unwrap();
    assert!(folder["size"].is_null());

    for (name, mime, content) in [
        (
            "hello%20world.txt",
            "text/plain",
            b"preview content".as_slice(),
        ),
        ("picture.png", "image/png", b"\x89PNG\r\n\x1a\n".as_slice()),
        (
            "document.pdf",
            "application/pdf",
            b"%PDF-1.7 fixture".as_slice(),
        ),
    ] {
        let response = test::call_service(
            &app,
            test::TestRequest::get()
                .uri(&format!("/get/file/{}/{name}", fixture.name))
                .to_request(),
        )
        .await;
        assert_eq!(response.status(), StatusCode::OK);
        assert!(
            response
                .headers()
                .get(header::CONTENT_TYPE)
                .unwrap()
                .to_str()
                .unwrap()
                .starts_with(mime)
        );
        assert!(response.headers().contains_key(header::CONTENT_DISPOSITION));
        assert_eq!(test::read_body(response).await.as_ref(), content);
    }
    let video = test::call_service(
        &app,
        test::TestRequest::get()
            .uri(&format!("/get/video/{}/clip.mp4", fixture.name))
            .insert_header((header::RANGE, "bytes=0-3"))
            .to_request(),
    )
    .await;
    assert_eq!(video.status(), StatusCode::PARTIAL_CONTENT);
    assert_eq!(test::read_body(video).await.as_ref(), b"0123");

    for (json, folder) in [(true, "json folder"), (false, "native folder")] {
        let mut request = test::TestRequest::post()
            .uri("/api/new_folder")
            .insert_header((
                header::CONTENT_TYPE,
                "multipart/form-data; boundary=spine-test-boundary",
            ));
        if json {
            request = request.insert_header((header::ACCEPT, "application/json"));
        }
        let response = test::call_service(
            &app,
            request
                .set_payload(multipart(
                    &[("path", &fixture.name), ("name", folder)],
                    None,
                ))
                .to_request(),
        )
        .await;
        assert_eq!(
            response.status(),
            if json {
                StatusCode::OK
            } else {
                StatusCode::SEE_OTHER
            }
        );
        if json {
            let data: Value = test::read_body_json(response).await;
            assert_eq!(data["ok"], true);
        } else {
            assert!(
                response
                    .headers()
                    .get(header::LOCATION)
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .starts_with("/#/")
            );
        }
        assert!(fixture.root.join(folder).is_dir());
    }
    for (json, filename) in [(true, "json-upload.txt"), (false, "native-upload.txt")] {
        let mut request = test::TestRequest::post()
            .uri("/upload_file")
            .insert_header((
                header::CONTENT_TYPE,
                "multipart/form-data; boundary=spine-test-boundary",
            ));
        if json {
            request = request.insert_header((header::ACCEPT, "application/json"));
        }
        let response = test::call_service(
            &app,
            request
                .set_payload(multipart(
                    &[("path", &fixture.name)],
                    Some((filename, "uploaded contents")),
                ))
                .to_request(),
        )
        .await;
        assert_eq!(
            response.status(),
            if json {
                StatusCode::OK
            } else {
                StatusCode::SEE_OTHER
            }
        );
        assert_eq!(
            fs::read_to_string(fixture.root.join(filename)).unwrap(),
            "uploaded contents"
        );
    }
    let refreshed = test::call_service(
        &app,
        test::TestRequest::get()
            .uri(&format!("/files/{}", fixture.name))
            .insert_header((header::ACCEPT, "application/json"))
            .to_request(),
    )
    .await;
    let refreshed: Value = test::read_body_json(refreshed).await;
    assert!(refreshed.as_array().unwrap().iter().any(|entry| {
        entry
            .get("File")
            .is_some_and(|file| file["name"] == "json-upload.txt")
    }));
    assert!(refreshed.as_array().unwrap().iter().any(|entry| {
        entry
            .get("Folder")
            .is_some_and(|folder| folder["name"] == "json folder")
    }));
    let response = test::call_service(
        &app,
        test::TestRequest::get()
            .uri(&format!("/files/{}/missing", fixture.name))
            .insert_header((header::ACCEPT, "application/json"))
            .to_request(),
    )
    .await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    assert!(test::read_body_json::<Value, _>(response).await["error"].is_string());
}

async fn wait_for(ds: &DataStore, key: &str, expected: Option<&str>) {
    tokio::time::timeout(std::time::Duration::from_secs(2), async {
        loop {
            if let Some(value) = ds.get(key.to_string()) {
                if expected.is_none_or(|expected| value == expected) {
                    return;
                }
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("datastore worker should apply queued writes");
}

#[actix_web::test]
async fn authentication_uses_cookie_session_and_reports_json_401_or_native_redirect() {
    let fixture = Fixture::new();
    let mut ds = DataStore::new();
    ds.listen();
    ds.send(Op::Upsert("Authenticate".into(), "true".into()));
    ds.send(Op::Upsert("Password".into(), "spine-test-password".into()));
    wait_for(&ds, "Password", Some("spine-test-password")).await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(ds.clone()))
            .wrap(
                SessionMiddleware::builder(CookieSessionStore::default(), Key::generate())
                    .cookie_secure(false)
                    .build(),
            )
            .configure(http_server::configure_public)
            .service(
                web::scope("")
                    .wrap(AuthMiddleware)
                    .configure(http_server::configure_protected_and_static),
            ),
    )
    .await;
    let peer: SocketAddr = "127.0.0.1:24001".parse().unwrap();
    for asset in [
        "/login",
        "/cdn/css/foundation.css",
        "/cdn/spine/files.css",
        "/cdn/spine/vendor/spine.min.js",
        "/cdn/spine/login.js",
    ] {
        let response = test::call_service(
            &app,
            test::TestRequest::get()
                .uri(asset)
                .peer_addr(peer)
                .to_request(),
        )
        .await;
        assert_eq!(
            response.status(),
            StatusCode::OK,
            "login assets must load without a session: {asset}"
        );
    }
    let uri = format!("/files/{}", fixture.name);
    let denied = test::call_service(
        &app,
        test::TestRequest::get()
            .uri(&uri)
            .peer_addr(peer)
            .insert_header((header::ACCEPT, "application/json"))
            .to_request(),
    )
    .await;
    assert_eq!(denied.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(
        test::read_body_json::<Value, _>(denied).await["login"],
        "/login"
    );
    let native = test::call_service(
        &app,
        test::TestRequest::get()
            .uri(&uri)
            .peer_addr(peer)
            .to_request(),
    )
    .await;
    assert_eq!(native.status(), StatusCode::SEE_OTHER);
    assert_eq!(native.headers().get(header::LOCATION).unwrap(), "/login");
    let wrong = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/login")
            .peer_addr(peer)
            .insert_header((header::ACCEPT, "application/json"))
            .insert_header((header::CONTENT_TYPE, "application/x-www-form-urlencoded"))
            .set_payload("password=wrong")
            .to_request(),
    )
    .await;
    assert_eq!(wrong.status(), StatusCode::UNAUTHORIZED);
    let logged_in = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/login")
            .peer_addr(peer)
            .insert_header((header::ACCEPT, "application/json"))
            .insert_header((header::CONTENT_TYPE, "application/x-www-form-urlencoded"))
            .set_payload("password=spine-test-password")
            .to_request(),
    )
    .await;
    assert_eq!(logged_in.status(), StatusCode::OK);
    let cookie = logged_in
        .response()
        .cookies()
        .next()
        .expect("session cookie")
        .into_owned();
    assert_eq!(
        test::read_body_json::<Value, _>(logged_in).await["ok"],
        true
    );
    wait_for(&ds, "127.0.0.1", None).await;
    let authenticated = test::call_service(
        &app,
        test::TestRequest::get()
            .uri(&uri)
            .peer_addr("127.0.0.1:24002".parse().unwrap())
            .cookie(cookie)
            .insert_header((header::ACCEPT, "application/json"))
            .to_request(),
    )
    .await;
    assert_eq!(
        authenticated.status(),
        StatusCode::OK,
        "login and middleware must normalize the peer IP consistently across ports"
    );
    for (password, location) in [
        ("wrong", "/login?toast=wrongpassword"),
        ("spine-test-password", "/#/files/"),
    ] {
        let native_login = test::call_service(
            &app,
            test::TestRequest::post()
                .uri("/login")
                .peer_addr(peer)
                .insert_header((header::CONTENT_TYPE, "application/x-www-form-urlencoded"))
                .set_payload(format!("password={password}"))
                .to_request(),
        )
        .await;
        assert_eq!(native_login.status(), StatusCode::SEE_OTHER);
        assert_eq!(
            native_login
                .headers()
                .get(header::LOCATION)
                .unwrap()
                .to_str()
                .unwrap(),
            location
        );
    }
}

#[actix_web::test]
async fn server_owned_spine_ui_uses_only_its_frozen_framework_runtime() {
    let app = test::init_service(
        App::new()
            .configure(http_server::configure_public)
            .configure(http_server::configure_protected_and_static),
    )
    .await;
    let page = test::call_service(&app, test::TestRequest::get().uri("/").to_request()).await;
    assert_eq!(page.status(), StatusCode::OK);
    let html = String::from_utf8(test::read_body(page).await.to_vec()).unwrap();
    for asset in [
        "vendor/spine.min.js",
        "files-api.js",
        "files.js",
        "app.js",
        "files.css",
    ] {
        assert!(
            html.contains(&format!("/cdn/spine/{asset}")),
            "server page must reference {asset}"
        );
    }
    for demo in [
        "BaseSpine",
        "FixtureFilesApi",
        "spine-files-api",
        "spine-assets",
        "counter",
        "app.min.js",
    ] {
        assert!(
            !html.contains(demo),
            "server page must not include upstream demo/configuration: {demo}"
        );
    }
    for asset in [
        "vendor/spine.js",
        "vendor/spine.min.js",
        "vendor/spine.min.js.map",
        "vendor/manifest.json",
        "app.js",
        "files-api.js",
        "files.js",
        "files.css",
        "login.js",
    ] {
        let response = test::call_service(
            &app,
            test::TestRequest::get()
                .uri(&format!("/cdn/spine/{asset}"))
                .to_request(),
        )
        .await;
        assert_eq!(response.status(), StatusCode::OK, "{asset}");
        let bytes = test::read_body(response).await;
        assert!(!bytes.is_empty(), "{asset}");
        if asset == "app.js" || asset == "files-api.js" || asset == "files.js" {
            let script = std::str::from_utf8(&bytes).unwrap();
            for demo in [
                "BaseSpine",
                "FixtureFilesApi",
                "spine-files-api",
                "spine-assets",
                "CounterExample",
                "example_table.json",
            ] {
                assert!(
                    !script.contains(demo),
                    "consumer-owned {asset} must not depend on {demo}"
                );
            }
        }
    }
    let manifest: Value =
        serde_json::from_str(include_str!("../src/www/vendor/spine/manifest.json")).unwrap();
    assert_eq!(manifest["name"], "SpineJS");
    assert_eq!(manifest["version"], "0.0.1");
    assert_eq!(
        manifest["sourceRevision"],
        "7121c4d1ab7e7d27c9b53e1296b73243e9494756"
    );
    for (name, bytes) in [
        (
            "spine.js",
            include_bytes!("../src/www/vendor/spine/spine.js").as_slice(),
        ),
        (
            "spine.min.js",
            include_bytes!("../src/www/vendor/spine/spine.min.js").as_slice(),
        ),
        (
            "spine.min.js.map",
            include_bytes!("../src/www/vendor/spine/spine.min.js.map").as_slice(),
        ),
    ] {
        assert_eq!(manifest["files"][name]["bytes"], bytes.len());
        let digest = manifest["files"][name]["sha256"].as_str().unwrap();
        assert_eq!(digest.len(), 64);
        assert!(digest.bytes().all(|byte| byte.is_ascii_hexdigit()));
    }
    for removed in [
        "missing",
        "app.min.js",
        "src/example_table.json",
        "style/css/main.css",
        "spine.min.js",
        "site.css",
    ] {
        let response = test::call_service(
            &app,
            test::TestRequest::get()
                .uri(&format!("/cdn/spine/{removed}"))
                .to_request(),
        )
        .await;
        assert_eq!(response.status(), StatusCode::NOT_FOUND, "{removed}");
    }
    let alias = test::call_service(&app, test::TestRequest::get().uri("/spine").to_request()).await;
    assert_eq!(alias.status(), StatusCode::PERMANENT_REDIRECT);
    assert_eq!(alias.headers().get(header::LOCATION).unwrap(), "/");
    let login = test::call_service(&app, test::TestRequest::get().uri("/login").to_request()).await;
    let login = String::from_utf8(test::read_body(login).await.to_vec()).unwrap();
    assert!(login.contains("/cdn/spine/vendor/spine.min.js"));
    assert!(login.contains("/cdn/spine/login.js"));
    assert!(!login.contains("/cdn/js/"));
    assert!(!html.contains("/cdn/js/"));
    for removed in [
        "app.js",
        "login.js",
        "navagation.js",
        "show_file.js",
        "table_builder.js",
        "upload.js",
    ] {
        let response = test::call_service(
            &app,
            test::TestRequest::get()
                .uri(&format!("/cdn/js/{removed}"))
                .to_request(),
        )
        .await;
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
