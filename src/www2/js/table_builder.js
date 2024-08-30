function table_builder() {
  let table_body = document.getElementById("table_body");
  table_body.innerHTML = "";

  let url = location.href.split(location.host)[1];
  let current_path = url.split("#/");
  current_path = current_path[current_path.length - 1];
  if (current_path.length > 0 || current_path == ".") {
    table_body.appendChild(create_table_row("..", "folder", "", ".."));
  }
  fetch("/files")
    .then((data) => data.json())
    .then((data) => {
      data.forEach((d) => {
        let file_name = d.File == undefined ? d.Folder.name : d.File.name;
        let file_type = d.File == undefined ? "folder" : "file";
        let file_path = d.File == undefined ? d.Folder.path : d.File.path;
        let file_created = d.File == undefined ? "" : d.File.created;
        let new_file_path = file_path.split("/");
        new_file_path.pop();
        new_file_path = new_file_path.toString();
        new_file_path = new_file_path.replace(/,/g, "/");
        if (
          new_file_path == current_path ||
          new_file_path == "." + current_path ||
          new_file_path == "./" + current_path + "/"
        ) {
          console.log(new_file_path, current_path);
          table_body.appendChild(
            create_table_row(file_name, file_type, file_created, file_path),
          );
        }
      });
    });
}

function goto_folder(path) {
  if (path == undefined) {
    return;
  }
  if (path == "..") {
    history.back();
  } else {
    window.location.href = "/#/" + path;
  }
}

function create_table_row(file_name, file_type, file_date, file_path) {
  let file_name_component = document.createElement("td");
  let file_date_component = document.createElement("td");

  /* Icon */
  let icon_img = document.createElement("img");
  icon_img.classList = "dashboard-table-image";
  if (file_type == "file") {
    icon_img.src = "/api/icons/file.png";
  } else {
    icon_img.src = "/api/icons/folder.png";
  }
  icon_img.dataset.goto = file_path;

  let icon_shrink = document.createElement("div");
  icon_shrink.classList = "flex-child-shrink";
  icon_shrink.dataset.goto = file_path;

  let icon_container = document.createElement("div");
  icon_container.classList = "flex-container align-justify align-top";

  icon_shrink.appendChild(icon_img);
  icon_container.appendChild(icon_shrink);

  /* File Name */
  let file_name_h6 = document.createElement("h6");
  file_name_h6.classList = "dashboard-table-text";
  file_name_h6.innerText = file_name;
  file_name_h6.dataset.goto = file_path;

  let file_name_container = document.createElement("div");
  file_name_container.classList = "flex-child-grow";

  file_name_container.dataset.goto = file_path;
  file_name_container.appendChild(file_name_h6);

  icon_container.appendChild(file_name_container);

  file_name_component.appendChild(icon_container);

  /* File Date */
  let timestamp = document.createElement("span");
  timestamp.classList = "dashboard-table-timestamp";
  timestamp.innerText = file_date;
  timestamp.dataset.goto = file_path;

  file_date_component.appendChild(timestamp);

  /* Row */

  let row = document.createElement("tr");
  row.appendChild(file_name_component);
  row.appendChild(file_date_component);

  row.dataset.goto = file_path;
  if (file_type == "folder") {
    row.onclick = function (e) {
      console.log(e.target.dataset.goto);
      goto_folder(e.target.dataset.goto);
    };
  } else {
    row.onclick = function (e) {
      console.log(e.target.dataset.goto);
      show_file(e.target.dataset.goto);
    };
  }

  return row;
}
