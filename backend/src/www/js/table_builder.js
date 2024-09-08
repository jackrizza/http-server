// <!-- <div class="table-container">
//     <table class="dashboard-table">
//         <colgroup>
//             <col width="400" />
//             <col width="200" />
//         </colgroup>
//         <thead>
//             <tr>
//                 <th>
//                     <a href="#">Name<i class="fa fa-caret-down"></i></a>
//                 </th>
//                 <th>
//                     <a href="#">Date<i class="fa fa-caret-down"></i></a>
//                 </th>
//             </tr>
//         </thead>
//         <tbody id="table_body"></tbody>
//     </table>
// </div> -->

const EXTENSIONS = [
  "csv",
  "png",
  "json",
  "pdf",
  "txt",
  "jpg",
  "js",
  "cpp",
  "cs",
  "html",
  "dwg",
  "sql",
  "php",
  "css",
];

function table_container() {
  let table_container = document.createElement("div");
  table_container.classList = "table-container";
  table_container.setAttribute("ondrop", "dropHandler(event);");
  table_container.setAttribute("ondragover", "dragOverHandler(event);");
  let table = document.createElement("table");
  table.classList = "dashboard-table";
  let colgroup = document.createElement("colgroup");
  let col1 = document.createElement("col");
  col1.width = "400";
  let col2 = document.createElement("col");
  col2.width = "200";
  colgroup.appendChild(col1);
  colgroup.appendChild(col2);
  let thead = document.createElement("thead");
  let tr = document.createElement("tr");
  let th1 = document.createElement("th");
  let a1 = document.createElement("a");
  a1.href = "#";
  a1.innerText = "Name";
  let i1 = document.createElement("i");
  i1.classList = "fa fa-caret-down";
  a1.appendChild(i1);
  th1.appendChild(a1);
  let th2 = document.createElement("th");
  let a2 = document.createElement("a");
  a2.href = "#";
  a2.innerText = "Date";
  let i2 = document.createElement("i");
  i2.classList = "fa fa-caret-down";
  a2.appendChild(i2);
  th2.appendChild(a2);
  tr.appendChild(th1);
  tr.appendChild(th2);
  thead.appendChild(tr);
  let tbody = document.createElement("tbody");
  tbody.id = "table_body";
  table.appendChild(colgroup);
  table.appendChild(thead);
  table.appendChild(tbody);
  table_container.appendChild(table);

  return table_container;
}

function table_builder() {
  let table_body = document.getElementById("table_body");
  table_body.innerHTML = "";

  let url = location.href.split(location.host)[1];
  let current_path = url.split("#/");
  current_path = current_path[current_path.length - 1];
  if (current_path.length > 0 || current_path == ".") {
    table_body.appendChild(create_table_row("..", "folder", "", ".."));
  }

  // console.log(current_path);
  fetch("/files/" + current_path)
    .then((data) => data.json())
    .then((data) => {
      data.forEach((d) => {
        let file_path = d.File == undefined ? d.Folder.path : d.File.path;
        let new_file_path = file_path.split("/");
        new_file_path.pop();
        new_file_path = new_file_path.toString();
        new_file_path = new_file_path.replace(/,/g, "/");
        if (match_path(new_file_path, current_path)) {
          let file_name = d.File == undefined ? d.Folder.name : d.File.name;
          let file_type = d.File == undefined ? "folder" : "file";
          let file_created = d.File == undefined ? "" : d.File.created;
          table_body.appendChild(
            create_table_row(file_name, file_type, file_created, file_path),
          );
        }
      });
    });
}

function match_path(new_file_path, current_path) {
  return (
    new_file_path == current_path ||
    new_file_path == "." + current_path ||
    new_file_path == "./" + current_path + "/"
  );
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
    icon_img.src = file_image(file_name);
  } else {
    icon_img.src = "/cdn/icons/folder.png";
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
      // console.log(e.target.dataset.goto);
      goto_folder(e.target.dataset.goto);
    };
  } else {
    row.onclick = function (e) {
      // console.log(e.target.dataset.goto);
      show_file(e.target.dataset.goto);
    };
  }

  return row;
}

function create_new_folder_row(current_path) {
  let file_name_component = document.createElement("td");
  let file_date_component = document.createElement("td");

  /* Icon */
  let icon_img = document.createElement("img");
  icon_img.classList = "dashboard-table-image";

  icon_img.src = "/cdn/icons/folder.png";

  let icon_shrink = document.createElement("div");
  icon_shrink.classList = "flex-child-shrink";

  let icon_container = document.createElement("div");
  icon_container.classList = "flex-container align-justify align-top";

  icon_shrink.appendChild(icon_img);
  icon_container.appendChild(icon_shrink);

  /* New Folder Name Form */
  let form = document.createElement("form");
  form.classList = "table-form";
  form.action = "/api/new_folder";
  form.enctype = "multipart/form-data";
  form.method = "POST";

  let new_folder_name = document.createElement("input");
  new_folder_name.type = "text";
  new_folder_name.placeholder = "New Folder Name";
  new_folder_name.name = "name";

  let new_folder_path = document.createElement("input");
  new_folder_path.type = "hidden";
  new_folder_path.name = "path";
  new_folder_path.value = current_path;

  let new_folder_submit = document.createElement("input");
  new_folder_submit.type = "submit";
  new_folder_submit.value = "create";

  let file_name_container = document.createElement("div");
  file_name_container.classList = "flex-child-grow";

  form.appendChild(new_folder_name);
  form.appendChild(new_folder_path);
  form.appendChild(new_folder_submit);
  file_name_container.appendChild(form);

  /* File Date */
  let cancel = document.createElement("a");
  cancel.classList = "dashboard-table-timestamp";
  cancel.innerText = "cancel";

  cancel.onclick = function (e) {
    e.preventDefault();
    e.stopPropagation();
    e.target.parentElement.parentElement.remove();
  };

  icon_container.appendChild(file_name_container);
  file_name_component.appendChild(icon_container);
  file_date_component.appendChild(cancel);

  /* Row */

  let row = document.createElement("tr");
  row.id = "new_folder_row";
  row.appendChild(file_name_component);
  row.appendChild(file_date_component);

  let table_body = document.getElementById("table_body");
  table_body.insertBefore(row, table_body.childNodes[0]);
}

function file_image(path) {
  let file_extension = path.split(".");
  file_extension = file_extension[file_extension.length - 1];

  if (EXTENSIONS.includes(file_extension)) {
    return "/cdn/icons/" + file_extension + ".png";
  }
  if (["jpeg", "heic"].includes(file_extension)) {
    return "/cdn/icons/img.png";
  }

  if (["mp4"].includes(file_extension)) {
    return "/cdn/icons/video.png";
  }

  return "/cdn/icons/file.png";
}
