$(document).foundation();

let pressed = false;
window.onload = function () {
  table_builder();

  document.getElementById("upload_file_modal_toggle_top_bar").addEventListener(
    "click",
    function (e) {
      if (pressed) {
        return;
      }
      e.preventDefault();
      upload_overlay();
      change_pressed();
    },
    false,
  );

  change_pressed = function () {
    pressed = !pressed;
    setTimeout(() => {
      pressed = !pressed;
    }, 125);
  };
};
window.onhashchange = function () {
  table_builder();
};
