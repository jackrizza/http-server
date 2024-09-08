// <div class="top-bar">
//     <div class="top-bar-left">
//         <ul class="dropdown menu" data-dropdown-menu>
//             <li class="menu-text"><h4>Simple HTTP Server</h4></li>
//         </ul>
//     </div>
//     <div class="top-bar-right">
//         <ul class="menu">
//             <li>
//                 <a href="" id="new_folder_modal_toggle_top_bar"
//                     >New Folder</a
//                 >
//             </li>
//             <li>
//                 <a href="" id="upload_file_modal_toggle_top_bar"
//                     >Upload File</a
//                 >
//             </li>
//         </ul>
//     </div>
// </div>

function topbar() {
  let topbar = document.createElement("div");
  topbar.classList = "top-bar";
  let tb_left = document.createElement("div");
  tb_left.classList = "top-bar-left";

  let tb_right = document.createElement("div");
  tb_right.classList = "top-bar-right";

  let right_menu = document.createElement("ul");
  right_menu.classList = "menu";

  let menu_text = document.createElement("li");
  menu_text.classList = "menu-text";
  let h4 = document.createElement("h4");
  h4.innerText = "Simple HTTP Server";
  menu_text.appendChild(h4);

  let menu_new_folder = document.createElement("li");
  let new_folder_btn = document.createElement("a");
  new_folder_btn.href = "";
  new_folder_btn.id = "new_folder_modal_toggle_top_bar";
  new_folder_btn.innerText = "New Folder";

  let menu_upload_file = document.createElement("li");
  let upload_file_btn = document.createElement("a");
  upload_file_btn.href = "";
  upload_file_btn.id = "upload_file_modal_toggle_top_bar";
  upload_file_btn.innerText = "Upload File";

  menu_new_folder.appendChild(new_folder_btn);
  menu_upload_file.appendChild(upload_file_btn);
  right_menu.appendChild(menu_new_folder);
  right_menu.appendChild(menu_upload_file);
  tb_right.appendChild(right_menu);
  topbar.appendChild(tb_left);
  topbar.appendChild(tb_right);

  upload_file_btn.addEventListener("click", function (e) {
    e.preventDefault();
    upload_overlay();
  });

  new_folder_btn.addEventListener("click", function (e) {
    e.preventDefault();

    let url_path = location.href.split(location.host)[1].split("#/");
    if (document.getElementById("new_folder_row") == undefined) {
      create_new_folder_row(url_path[1]);
    }
  });

  return topbar;
}
