window.onload = function () {
  let url = location.href.split(location.host)[1];
  let toast = url.split("?");
  if (toast.length > 1) {
    let msg = toast[1].split("=");
    alert(msg[0], msg[1]);
  }
};

function alert(type, msg) {
  let alert = document.createElement("div");
  alert.classList = "callout";
  alert.classList.add("alert");
  alert.classList.add("callout-" + type);
  if (msg == "wrongpassword") {
    alert.innerText = "Wrong Password. Please try again.";
  }

  let container = document.createElement("div");
  container.classList = "callout-container"; //grid-container
  container.appendChild(alert);

  document.body.appendChild(container);

  setTimeout(function () {
    alert.remove();
  }, 5000);
}
