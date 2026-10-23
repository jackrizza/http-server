// Passwords and sessions are never persisted by Spine.
const loginState = new State({ persist: false, initial: {
    pending: false,
    error: new URLSearchParams(location.search).get("toast") === "wrongpassword"
        ? "Wrong password. Please try again." : ""
} });

function loginDestination() {
    const next = new URLSearchParams(location.search).get("next")
        || (location.hash.startsWith("#/") ? `/${location.hash}` : "");
    return next?.startsWith("/#/") ? next : "/#/files/";
}

const loginForm = new Form({ classlist: ["login"],
    attrs: { method: "post", action: "/login", "aria-label": "Log in" },
    children: [
        new H1({ plainText: "Simple HTTP Server" }),
        new P({ plainText: "Enter the server password to browse and share files." }),
        new Label({ plainText: "Password", attrs: { for: "server-password" } }),
        new Input({ key: "server-password", options: { type: "password", name: "password" },
            attrs: { required: true, autocomplete: "current-password", autofocus: true } }),
        new P({ store: loginState, watch: ["error"],
            plainText: state => state.get("error"), attrs: { role: "alert" } }),
        new Button({ classlist: ["button", "expanded"], store: loginState, watch: ["pending"],
            plainText: state => state.get("pending") ? "Logging in…" : "Log in",
            attrs: { type: "submit", disabled: state => state.get("pending") } })
    ],
    on: { submit: async event => {
        event.preventDefault();
        if (loginState.get("pending")) return;
        const password = event.currentTarget.elements.namedItem("password");
        const body = new URLSearchParams({ password: password.value });
        loginState.set("pending", true);
        loginState.set("error", "");
        try {
            await request("/login", { method: "POST", body, credentials: "same-origin",
                headers: { Accept: "application/json" }, responseType: "json" });
            password.value = "";
            location.assign(loginDestination());
        } catch (error) {
            loginState.set("error", error.status === 401 ? "Wrong password. Please try again."
                : "Could not log in. Please try again.");
            password.value = "";
            password.focus();
        } finally {
            loginState.set("pending", false);
        }
    } }
});
loginForm.mount(document.getElementById("app"));
