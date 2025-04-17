const CODE_EXTENSIONS = [
  "js",
  "rs",
  "py",
  "java",
  "c",
  "cpp",
  "html",
  "css",
  "scss",
  "json",
  "xml",
  "yaml",
  "yml",
  "sh",
  "sql",
  "php",
  "rb",
  "go",
  "ts",
  "tsx",
  "jsx",
  "md",
  "markdown",
  "csv",
  "txt",
  "log",
  "conf",
  "ini",
  "cfg",
  "env",
  "htaccess",
  "gitignore",
  "dockerignore",
  "dockerfile",
  "gitattributes",
  "gitmodules",
  "gitconfig",
  "gitkeep",
  "gitlog",
  "gitmessage",
];

const VIDEO_EXTENSIONS = [
  "mp4",
  "mkv",
  "webm",
  "avi",
  "mov",
  "MOV",
  "wmv",
  "flv",
  "mpeg",
  "mpg",
  "3gp",
  "3g2",
  "m4v",
]

function show_file(path) {
  if (path == undefined) {
    return;
  }

  let frame = document.createElement("code");
  frame.id = "file_preview";
  let ext = path.split(".");
  ext = ext[ext.length - 1];
  if (CODE_EXTENSIONS.includes(ext)) {
    frame.classList = `prettyprint`;
    frame.innerText = "Loading...";
    /* File */
    fetch("/get/file/" + path)
      .then((data) => data.text())
      .then((data) => {
        console.log(data);
        document.getElementById("file_preview").innerText = data;
        PR.prettyPrint();
      });
  } else if (VIDEO_EXTENSIONS.includes(ext)) {
    /* Video */
    const cleanPath = path.replace(/^\.\//, "");

    frame = document.createElement("video");
    frame.className = "file-frame";
    frame.controls = true;
    frame.preload = "none";             // ← only metadata
    // (optional) use <source>:
    const source = document.createElement("source");
    source.src = `/get/video/${encodeURIComponent(cleanPath)}`;
    // source.type = `video/${ext}`;
    frame.appendChild(source);

    // update download link to hit your video route

  } else {
    /* File */
    frame = document.createElement("object");
    frame.classList = "file-frame";
    frame.data = "/get/file/" + path;
    frame.type = "text/plain";
    switch (ext) {
      case "pdf":
        frame.type = "application/pdf";
        break;
      case "png":
        frame.type = "image/png";
        break;
      case "jpg":
      case "jpeg":
        frame.type = "image/jpeg";
        break;
      case "csv":
        frame.type = "text/csv";
        break;
      case "json":
        frame.type = "application/json";
        break;
      case "js":
        frame.type = "application/javascript";
        break;
    }
  }

  /* Top Bar */
  let topbar = document.createElement("div");
  topbar.classList = "top-bar overlay-top-bar";
  let tb_left = document.createElement("div");
  tb_left.classList = "top-bar-left";

  let tb_right = document.createElement("div");
  tb_right.classList = "top-bar-right";

  let right_menu = document.createElement("ul");
  right_menu.classList = "menu";

  let menu_download = document.createElement("li");
  let download_btn = document.createElement("a");
  download_btn.href = "/get/file/" + path;
  download_btn.download = path;
  download_btn.innerText = "Download";

  menu_download.appendChild(download_btn);
  right_menu.appendChild(menu_download);
  tb_right.appendChild(right_menu);
  topbar.appendChild(tb_left);
  topbar.appendChild(tb_right);

  let div = document.createElement("div");
  div.classList = "overlay";
  div.onclick = function (e) {
    if (e.target.classList.contains("overlay")) {
      e.target.remove();
    }
  };

  div.appendChild(topbar);
  div.appendChild(frame);

  document.body.appendChild(div);

  if (frame.tagName === "VIDEO") {
    frame.load();
    frame.preload = "metadata";
    frame.addEventListener("loadedmetadata", () => {
      // Jump to a tiny fraction into the file…
      frame.currentTime = 0.1;
    });
  }
}
