function show_file(path) {
  if (path == undefined) {
    return;
  }

  let ext = path.split(".");
  ext = ext[ext.length - 1];

  /* File */
  let frame = document.createElement("object");
  frame.classList = "file-frame";
  frame.data = "/get/file/" + path;
  frame.type = "text/plain";
  if (ext == "pdf") {
    frame.type = "application/pdf";
  }
  if (ext == "png") {
    frame.type = "image/png";
  }
  if (ext == "jpg" || ext == "jpeg") {
    frame.type = "image/jpeg";
  }
  if (ext == "csv") {
    frame.type = "text/csv";
  }
  if (ext == "json") {
    frame.type = "application/json";
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
}
