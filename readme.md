# Simple HTTP Server

A small Rust server for browsing, sharing, and uploading files from the current
working directory. The browser UI uses SpineJS v0.0.1 and is embedded in the
executable, including its styles and framework runtime.

This project is intended for quick file sharing, not production hosting.

## Run

```sh
cargo run -p cli
```

Open `http://localhost:8888/`. To choose a different HTTP port:

```sh
cargo run -p cli -- --port 8000
```

Build the executable with `cargo build -p cli`, then run it from the directory
you want to share. The binary is `target/debug/cli.exe` on Windows and
`target/debug/cli` on Unix systems. No Node.js runtime or frontend build is
required to run the server.

## File browser

The UI provides a folder sidebar, filename search, sorting by name, size, or date,
folder creation, uploads (including drag and drop), and downloads. Text, image,
PDF, and video previews open in a modal with a download link. Close a preview with
Escape, the close button, or a click on the backdrop. Videos use range requests.

![File browser](screenshots/landing.jpg)

![File preview modal](screenshots/preview.jpg)

The screenshots use sample files. The server shows the contents of the directory
from which you launch it.

## Authentication and HTTPS

Generate a local certificate and key with OpenSSL:

```sh
openssl req -x509 -newkey rsa:4096 -nodes -keyout key.pem -out cert.pem -days 365 -subj '/CN=localhost'
```

Start the server with a password and the certificate paths:

```sh
cargo run -p cli -- --authenticate --password your-password --pem-file key.pem --cert-file cert.pem
```

Authenticated mode listens on HTTPS port `8443`; `--port` applies to plain HTTP.
Login uses an encrypted, HTTP-only cookie session. Passwords are not stored in
Spine's persistent browser state. OpenSSL is needed only to generate the
certificate; the server uses Rustls at runtime.

![Login](screenshots/login.jpg)

## Development

```sh
node --test backend/tests/frontend.test.cjs
cargo test -p backend
cargo build --workspace
```

Node.js is needed only for the frontend tests. The Actix integration tests use
local temporary fixtures and do not require a running server.

For a separate loopback preview host:

```sh
cargo run -p backend --example spine_preview -- 8889
```

Open `http://127.0.0.1:8889/`. This example uses the same embedded UI and routes
with authentication disabled. Rebuild and restart after frontend changes because
Rust embeds the assets at compile time.

| Location | Purpose |
| --- | --- |
| `backend/` | HTTP handlers, authentication, embedded frontend, and tests |
| `backend/src/www/spine-ui/` | Application components, styles, and API adapter |
| `backend/src/www/vendor/spine/` | Pinned framework-only SpineJS release assets |
| `cli/` | Command-line server launcher |
| `gui/` | Native tray application that launches the CLI |
| `screenshots/` | Current browser, login, and modal screenshots |

## Framework and API

The frontend uses [SpineJS v0.0.1](https://github.com/jackrizza/spinejs/releases/tag/v0.0.1).
The vendored manifest records the release commit, artifact URLs, sizes, and
verified SHA-256 digests. See the [runtime upgrade instructions](backend/src/www/vendor/spine/README.md).
The application has no dependency on the upstream demo or a separate checkout.

| Route | Contract |
| --- | --- |
| `GET /files/{path}` | JSON `{ File: ... }` / `{ Folder: ... }` entries with file sizes |
| `GET /get/file/{path}` | File contents for previews and downloads |
| `GET /get/video/{path}` | Media contents with range support |
| `POST /upload_file` | Multipart `path`, `file` |
| `POST /api/new_folder` | Multipart `path`, `name` |
| `POST /login` | Password form and cookie session |

Mutation and login requests accepting `application/json` receive JSON responses.
Protected JSON requests receive `401` when authentication is required. Native form
posts retain redirect responses.

The browser lives at `/`, with folder routes such as `/#/files/documents`.
`/spine` redirects to `/`, and older `/#/<folder>` bookmarks are translated to the
current routes. `/cdn/spine/` serves the application and vendored runtime; the
legacy `/cdn/js/` routes have been removed.
