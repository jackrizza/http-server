// This server owns its endpoints, file formats, and preview rules.
function filesPath(path = "") {
    return String(path).replace(/\\/g, "/").replace(/^\.\//, "").split("/")
        .filter(part => part && part !== ".").join("/");
}

function filesEncodedPath(path) {
    return filesPath(path).split("/").map(encodeURIComponent).join("/");
}

function normalizeFileEntries(entries, parent = "") {
    if (!Array.isArray(entries)) throw new TypeError("Directory response must be an array");
    parent = filesPath(parent);
    return entries.map(entry => {
        const value = entry.File || entry.Folder;
        if (!value || typeof value.name !== "string") throw new TypeError("Invalid File/Folder entry");
        const path = filesPath(value.path || [parent, value.name].filter(Boolean).join("/"));
        return { name: value.name, path, created: String(value.created || ""),
            kind: entry.Folder ? "folder" : "file", size: value.size };
    }).filter(entry => entry.path.split("/").slice(0, -1).join("/") === parent);
}

function filePreviewKind(name) {
    const extension = String(name).split(".").pop().toLowerCase();
    if (["png", "jpg", "jpeg", "gif", "webp", "svg", "avif"].includes(extension)) return "image";
    if (extension === "pdf") return "pdf";
    if (["mp4", "mkv", "webm", "ogg", "avi", "mov", "wmv", "flv", "mpeg", "mpg", "3gp", "3g2", "m4v"].includes(extension)) return "video";
    if (["txt", "md", "markdown", "json", "js", "css", "html", "csv", "rs", "xml", "log", "py", "java", "c", "cpp", "cs", "scss", "yaml", "yml", "sh", "sql", "php", "rb", "go", "ts", "tsx", "jsx", "conf", "ini", "cfg", "env", "htaccess", "gitignore", "dockerignore", "dockerfile", "gitattributes", "gitmodules", "gitconfig", "gitkeep", "gitlog", "gitmessage"].includes(extension)) return "text";
    return "download";
}

async function filesRequest(url, options = {}) {
    try {
        return await request(url, { cache: false, credentials: "same-origin", ...options,
            headers: { Accept: "application/json", ...options.headers } });
    } catch (error) {
        if (error.status === 401) {
            location.assign(`/login?next=${encodeURIComponent(`/${location.hash || "#/files/"}`)}`);
        }
        if (error.response?.headers.get("content-type")?.includes("application/json")) {
            const detail = await error.response.json().catch(() => null);
            if (detail?.error) error.message = detail.error;
        }
        throw error;
    }
}

class HttpFilesApi {
    constructor({ baseUrl = "", nativeForms = false } = {}) {
        this.baseUrl = baseUrl.replace(/\/$/, "");
        this.nativeForms = nativeForms;
        this.forms = { upload: `${this.baseUrl}/upload_file`, newFolder: `${this.baseUrl}/api/new_folder` };
    }

    list(path, { signal } = {}) {
        return filesRequest(`${this.baseUrl}/files/${filesEncodedPath(path)}`, {
            responseType: "json", credentials: "same-origin", headers: { Accept: "application/json" }, cache: false, signal
        }).then(entries => normalizeFileEntries(entries, path));
    }

    fileUrl(entry) {
        const endpoint = filePreviewKind(entry.name) === "video" ? "/get/video/" : "/get/file/";
        return `${this.baseUrl}${endpoint}${filesEncodedPath(entry.path)}`;
    }

    async preview(entry, { signal } = {}) {
        const kind = filePreviewKind(entry.name);
        const url = this.fileUrl(entry);
        // Let media use the server's range support instead of buffering whole videos.
        if (kind === "download" || kind === "video") return { kind, url };
        const result = await filesRequest(url, { signal, cache: false, credentials: "same-origin",
            responseType: kind === "text" ? "text" : "blob" });
        return kind === "text" ? { kind, text: result } : { kind, blob: result };
    }

    upload(path, file, { signal } = {}) {
        const body = new FormData();
        body.append("path", filesPath(path));
        body.append("file", file);
        return this._mutate(this.forms.upload, body, signal);
    }

    createFolder(path, name, { signal } = {}) {
        const body = new FormData();
        body.append("path", filesPath(path));
        body.append("name", name);
        return this._mutate(this.forms.newFolder, body, signal);
    }

    _mutate(url, body, signal) {
        // Opt-in JSON is the target integration contract. Legacy servers can use
        // nativeForms:true and retain their existing redirect behavior.
        return filesRequest(url, { method: "POST", body, signal, cache: false,
            credentials: "same-origin", headers: { Accept: "application/json" }, responseType: "json" });
    }
}

function defaultFilesApi() { return new HttpFilesApi(); }
