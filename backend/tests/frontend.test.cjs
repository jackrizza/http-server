const { test } = require("node:test");
const assert = require("node:assert/strict");
const { readFileSync } = require("node:fs");
const { join } = require("node:path");
const { createHash } = require("node:crypto");
const vm = require("node:vm");
const www = join(__dirname, "../src/www");

function runtime(fetch = async () => new Response("[]", { headers: { "content-type": "application/json" } })) {
    const location = { hash: "#/files/reports", assign(url) { this.destination = url; } };
    const context = vm.createContext({ fetch, location, Headers, Response, FormData, Blob,
        AbortController, URLSearchParams, console, window: { navigator: { onLine: true } } });
    for (const file of ["vendor/spine/spine.min.js", "spine-ui/files-api.js", "spine-ui/files.js"]) {
        vm.runInContext(readFileSync(join(www, file), "utf8"), context);
    }
    return { context, run: code => vm.runInContext(code, context) };
}

test("vendored bundles match the pinned release digests", () => {
    const manifest = JSON.parse(readFileSync(join(www, "vendor/spine/manifest.json"), "utf8").replace(/^\uFEFF/, ""));
    assert.equal(manifest.version, "0.0.1");
    for (const [name, metadata] of Object.entries(manifest.files)) {
        const bytes = readFileSync(join(www, "vendor/spine", name));
        assert.equal(bytes.length, metadata.bytes);
        assert.equal(createHash("sha256").update(bytes).digest("hex"), metadata.sha256);
    }
});

test("the listing adapter handles Windows paths, nested folders, and encoded filenames", async () => {
    let requested;
    const { run } = runtime(async (url, options) => {
        requested = { url, options };
        return Response.json([
            { File: { name: "<draft> #1.txt", path: "./reports/2026/<draft> #1.txt" } },
            { Folder: { name: "nested", path: "./reports/2026/nested" } },
            { File: { name: "other.txt", path: "./elsewhere/other.txt" } }
        ]);
    });
    const entries = await run('new HttpFilesApi().list("reports\\\\2026")');
    assert.equal(entries.length, 2);
    assert.equal(entries[0].name, "<draft> #1.txt");
    assert.equal(entries[1].kind, "folder");
    assert.equal(requested.url, "/files/reports/2026");
    assert.equal(requested.options.headers.get("Accept"), "application/json");
    assert.equal(run('new HttpFilesApi().fileUrl({ name: "<draft> #1.txt", path: "reports/<draft> #1.txt" })'),
        "/get/file/reports/%3Cdraft%3E%20%231.txt");
});

test("video previews stream without fetching a whole Blob; source previews stay text", async () => {
    let requests = 0;
    const { run } = runtime(async () => { requests++; return new Response("<script>example</script>"); });
    const video = await run('new HttpFilesApi().preview({ name: "movie.MP4", path: "media/movie.MP4" })');
    assert.equal(video.kind, "video");
    assert.equal(video.url, "/get/video/media/movie.MP4");
    assert.equal(requests, 0);
    const source = await run('new HttpFilesApi().preview({ name: "source.py", path: "source.py" })');
    assert.equal(source.kind, "text");
    assert.equal(source.text, "<script>example</script>");
    assert.equal(requests, 1);
});

test("mutations use multipart fields and report server errors", async () => {
    const calls = [];
    const { context, run } = runtime(async (url, options) => {
        calls.push({ url, options });
        if (url.endsWith("new_folder")) return Response.json({ error: "Folder cannot be created" }, { status: 400 });
        return Response.json({ ok: true });
    });
    context.file = new Blob(["uploaded contents"], { type: "text/plain" });
    await run('new HttpFilesApi().upload("reports", file)');
    assert.equal(calls[0].options.body.get("path"), "reports");
    assert.equal(await calls[0].options.body.get("file").text(), "uploaded contents");
    assert.equal(calls[0].options.method, "POST");
    await assert.rejects(run('new HttpFilesApi().createFolder("reports", "New folder")'), /Folder cannot be created/);
    assert.equal(calls[1].options.body.get("name"), "New folder");
});

test("expired API sessions return through login to the current directory", async () => {
    const { context, run } = runtime(async () => Response.json({ error: "Authentication required" }, { status: 401 }));
    await assert.rejects(run('new HttpFilesApi().list("reports")'), /Authentication required/);
    assert.equal(context.location.destination, "/login?next=%2F%23%2Ffiles%2Freports");
});

test("late directory results cannot overwrite a newer navigation", async () => {
    const { context, run } = runtime();
    let finishOld;
    context.api = { forms: {}, list: path => path === "old"
        ? new Promise(resolve => { finishOld = resolve; }) : Promise.resolve([{ path: "new/file.txt", name: "file.txt" }]) };
    run("const browserUnderTest = new FileBrowser({ api }); const oldLoad = browserUnderTest.navigate('old');");
    await run("browserUnderTest.navigate('new')");
    finishOld([{ path: "old/stale.txt", name: "stale.txt" }]);
    await run("oldLoad");
    assert.equal(run("browserUnderTest.state.get('entries')[0].path"), "new/file.txt");
    assert.equal(run("browserUnderTest.state.get('loading')"), false);
});

test("dropped files upload sequentially to the current folder and refresh once", async () => {
    const { context, run } = runtime();
    const calls = [];
    context.api = { forms: {}, upload: async (path, file) => calls.push([path, file.name]),
        list: async path => { calls.push(["list", path]); return []; } };
    await run("const browserUnderTest = new FileBrowser({ api, path: 'reports' }); browserUnderTest.uploadFiles([{ name: 'one.txt' }, { name: 'two.txt' }])");
    assert.deepEqual(calls, [["reports", "one.txt"], ["reports", "two.txt"], ["list", "reports"]]);
    assert.equal(run("browserUnderTest.state.get('busy')"), false);
});

test("search filters filenames without changing directory data and size sorting keeps folders first", () => {
    const { run } = runtime();
    run(`const browserUnderTest = new FileBrowser();
        browserUnderTest.state.set('entries', [
            { name: 'Report 10.txt', path: 'Report 10.txt', kind: 'file', size: 100 },
            { name: 'Documents', path: 'Documents', kind: 'folder' },
            { name: 'Report 2.txt', path: 'Report 2.txt', kind: 'file', size: 5 }
        ]);`);
    assert.equal(run("browserUnderTest._sorted(browserUnderTest.state)[1].name"), "Report 2.txt");
    run("browserUnderTest.state.set('query', 'REPORT 10')");
    assert.equal(run("browserUnderTest._sorted(browserUnderTest.state).length"), 1);
    assert.equal(run("browserUnderTest.state.get('entries').length"), 3);
    run("browserUnderTest.state.set('query', ''); browserUnderTest.sort('size')");
    assert.equal(run("browserUnderTest._sorted(browserUnderTest.state)[0].kind"), "folder");
    assert.equal(run("browserUnderTest._sorted(browserUnderTest.state)[1].size"), 5);
    run("browserUnderTest.sort('size')");
    assert.equal(run("browserUnderTest._sorted(browserUnderTest.state)[1].size"), 100);
});

test("file presentation handles empty files, missing metadata, and extension colors", () => {
    const { run } = runtime();
    assert.equal(run("formatFileSize(0)"), "0 B");
    assert.equal(run("formatFileSize(1536)"), "1.5 KB");
    assert.equal(run("formatFileSize(undefined)"), "—");
    assert.equal(run("formatFileDate('not a date')"), "—");
    assert.equal(run("fileIconKind('Data.xlsx')"), "sheet-icon");
    assert.equal(run("fileIconLabel('Photo.jpg')"), "JPEG");
});

test("legacy hashes migrate while Spine hashes retain their encoded paths", () => {
    const { context, run } = runtime();
    context.window.addEventListener = () => {};
    context.history = { replaceState(_state, _title, hash) { context.location.hash = hash; } };
    // Evaluate the migration function independently of application mounting.
    const app = readFileSync(join(www, "spine-ui/app.js"), "utf8");
    vm.runInContext(app.slice(0, app.indexOf("migrateFileHash();")), context);
    for (const [oldHash, expected] of [["", "#/files/"], ["#/.", "#/files/"],
        ["#/reports/hello%20world", "#/files/reports/hello%20world"],
        ["#/files/reports", "#/files/reports"]]) {
        context.location.hash = oldHash;
        run("migrateFileHash()");
        assert.equal(context.location.hash, expected);
    }
});

test("login submits the password, clears it, and returns to a local directory", async () => {
    let submitted;
    const { context, run } = runtime(async (url, options) => {
        submitted = { url, options };
        return Response.json({ ok: true });
    });
    context.location.search = "?next=%2F%23%2Ffiles%2Freports";
    context.document = { getElementById: () => null };
    run("Form.prototype.mount = function() {};");
    vm.runInContext(readFileSync(join(www, "spine-ui/login.js"), "utf8"), context);
    const password = { value: "fixture-password", focus() {} };
    context.event = { preventDefault() {}, currentTarget: { elements: { namedItem: () => password } } };
    await run("loginForm.on.submit(event)");
    assert.equal(submitted.url, "/login");
    assert.equal(submitted.options.body.get("password"), "fixture-password");
    assert.equal(submitted.options.credentials, "same-origin");
    assert.equal(password.value, "");
    assert.equal(context.location.destination, "/#/files/reports");
    assert.equal(run("loginState.get('pending')"), false);
    context.location.search = "?next=https%3A%2F%2Fexample.com";
    assert.equal(run("loginDestination()"), "/#/files/");
    context.location.search = "";
    context.location.hash = "#/files/nested";
    assert.equal(run("loginDestination()"), "/#/files/nested");
});

test("rejected login stays on the page, clears the password, and permits retry", async () => {
    const { context, run } = runtime(async () => Response.json({ error: "Invalid password" }, { status: 401 }));
    context.document = { getElementById: () => null };
    context.location.search = "";
    run("Form.prototype.mount = function() {};");
    vm.runInContext(readFileSync(join(www, "spine-ui/login.js"), "utf8"), context);
    let focused = false;
    const password = { value: "wrong", focus() { focused = true; } };
    context.event = { preventDefault() {}, currentTarget: { elements: { namedItem: () => password } } };
    await run("loginForm.on.submit(event)");
    assert.equal(password.value, "");
    assert.equal(focused, true);
    assert.equal(context.location.destination, undefined);
    assert.equal(run("loginState.get('error')"), "Wrong password. Please try again.");
    assert.equal(run("loginState.get('pending')"), false);
});
