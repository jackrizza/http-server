function show_file(path) {
  /* File */
  let frame = document.createElement("embed");
  frame.classList = "file-frame";
  frame.src = "/get/file/" + path;

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
