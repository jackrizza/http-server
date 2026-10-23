// The HTTP server owns this application. Spine supplies only the UI runtime.
// Translate existing bookmarks and native form redirects to Spine routes.
function migrateFileHash() {
    const hash = location.hash.slice(1);
    if (hash === "/files" || !hash || !hash.startsWith("/files/")) {
        const path = hash === "/files" ? "" : hash.replace(/^\//, "");
        history.replaceState(null, "", `#/files/${path === "." ? "" : path}`);
    }
}
migrateFileHash();
window.addEventListener("hashchange", migrateFileHash);
const fileBrowser = new FileBrowser({ api: new HttpFilesApi() });
const fileBrowserApp = new Spine([{
    key: "/files/*path",
    tree: new Tree("files", fileBrowser),
    update: context => fileBrowser.navigate(context.params.path || "", { signal: context.signal })
}], { root: "#app", defaultRoute: "/files/" });
fileBrowserApp.start();
