function upload_overlay() {
  /* Upload Input */
  let upload_input = document.createElement("input");
  upload_input.type = "file";
  upload_input.name = "payload";

  let upload_group = document.createElement("div");
  upload_group.classList = "input-group";
  upload_group.appendChild(upload_input);

  /* Submit Input */
  let submit_input = document.createElement("input");
  submit_input.type = "submit";
  submit_input.classList = "button expanded";
  submit_input.value = "Upload File";

  let submit_group = document.createElement("div");
  submit_group.classList = "input-group";
  submit_group.appendChild(submit_input);

  /* Hidden Path Input */
  let path_input = document.createElement("input");
  path_input.type = "hidden";
  path_input.name = "path";

  let url = location.href.split(location.host)[1];
  let current_path = url.split("#/");
  path_input.value = current_path[current_path.length - 1];

  /* Form */
  let heading = document.createElement("h4");
  heading.innerText = "Upload";

  let form = document.createElement("form");
  form.action = "/upload_file";
  form.method = "post";
  form.enctype = "multipart/form-data";
  form.appendChild(heading);
  form.appendChild(path_input);
  form.appendChild(upload_group);
  form.appendChild(submit_group);

  /* Overlay */
  let form_overlay = document.createElement("div");
  form_overlay.classList = "translucent-form-overlay";
  form_overlay.appendChild(form);

  let overlay = document.createElement("div");
  overlay.classList = "overlay";

  overlay.appendChild(form_overlay);

  overlay.onclick = function (e) {
    if (e.target.classList.contains("overlay")) {
      e.target.remove();
    }
  };

  document.body.appendChild(overlay);
}
