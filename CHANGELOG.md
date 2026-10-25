# Changelog

## 0.2.1

- Support uploads when the system temporary directory and shared folder are on
  different filesystems, including different Windows drives.
- Preserve the exact vendored SpineJS bytes when checking out on Windows.

## 0.2.0

- Package the CLI and backend as the single `jack-http-server` crate.
- Install the `http-server` command with Cargo and serve the launch directory.
- Embed the file browser, login UI, styles, and SpineJS runtime in the executable.
- Verify the standalone command and package on Linux, macOS, and Windows in CI.
- Publish tagged releases to crates.io and GitHub.
- License the project under MIT.
