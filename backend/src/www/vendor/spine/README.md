# SpineJS v0.0.1

The server embeds the framework-only assets from
https://github.com/jackrizza/spinejs/releases/tag/v0.0.1.
`manifest.json` records the release commit, download URLs, byte sizes, and SHA-256
digests verified against GitHub's release metadata. The application and login UI
live in `../../spine-ui/`; no upstream demo assets are included.

Rust embeds these files during compilation. Node.js and a separate SpineJS
checkout are not needed to build or deploy the server.

To upgrade, download all three framework assets (`spine.js`, `spine.min.js`, and
`spine.min.js.map`) from the desired release. Verify their digests against its
release metadata and update the manifest together with the files. Retain any
upstream notices and check the application's compatibility with the new runtime.

Run `cargo test --locked` and `cargo build --locked` after an upgrade, then check
login, navigation, sorting, uploads, folder creation, and previews in a browser.
