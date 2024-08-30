$(document).foundation();

let pressed = false;
window.onload = function () {
  table_builder();
  document.getElementById("upload_file_modal_toggle_top_bar").addEventListener(
    "click",
    function (e) {
      e.preventDefault();
      if (pressed) {
        return;
      }
      change_pressed();
      document.getElementById("upload_overlay").classList.toggle("hide");
    },
    false,
  );
  document.getElementById("upload_file_modal_toggle_overlay").addEventListener(
    "click",
    function (e) {
      e.preventDefault();

      if (pressed) {
        return;
      }
      change_pressed();
      document.getElementById("upload_overlay").classList.toggle("hide");
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
