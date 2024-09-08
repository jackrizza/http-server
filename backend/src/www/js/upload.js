function upload_overlay() {
  /* Upload Input */
  let upload_input = document.createElement("input");
  upload_input.type = "file";
  upload_input.name = "file";

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

function dropHandler(ev) {
  console.log("File(s) dropped");

  // Prevent default behavior (Prevent file from being opened)
  ev.preventDefault();

  if (ev.dataTransfer.items) {
    // Use DataTransferItemList interface to access the file(s)
    [...ev.dataTransfer.items].forEach((item, i) => {
      // If dropped items aren't files, reject them
      if (item.kind === "file") {
        const file = item.getAsFile();
        console.log(`… file[${i}].name = ${file.name}`);
        hiddenFormUpload(file);
      }
    });
  } else {
    // Use DataTransfer interface to access the file(s)
    [...ev.dataTransfer.files].forEach((file, i) => {
      console.log(`… file[${i}].name = ${file.name}`);
      hiddenFormUpload(file);
    });
  }
}

function dragOverHandler(ev) {
  console.log("File(s) in drop zone", ev);

  document.querySelector(".table-container").classList.add("dragover");

  // Prevent default behavior (Prevent file from being opened)
  ev.preventDefault();
}

function hiddenFormUpload(file) {
  // create a shadow form with the file input
  let url = location.href.split(location.host)[1];
  let current_path = url.split("#/");

  let data = new FormData();
  console.log(file.name);
  data.append("file", file);
  data.append("path", current_path[current_path.length - 1]);

  fetch("/upload_file", {
    method: "POST",
    enctype: "multipart/form-data",
    body: data,
  }).then(() => {
    console.log("File uploaded");
    location.reload();
  });
}
