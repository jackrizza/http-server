window.onload = function () {
  let app = document.getElementById("app");
  app.appendChild(topbar());

  app.appendChild(table_container());
  table_builder();
};
window.onhashchange = function () {
  table_builder();
};
