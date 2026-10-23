function fileIconLabel(name = "") {
  const extension = name.split(".").pop().toUpperCase();
  return extension === "JPG"
    ? "JPEG"
    : extension === "XLSX"
      ? "XLS"
      : extension.slice(0, 4);
}

function fileIconKind(name = "") {
  const extension = name.split(".").pop().toLowerCase();
  if (["xls", "xlsx", "csv"].includes(extension)) return "sheet-icon";
  if (filePreviewKind(name) === "image") return "image-icon";
  if (extension === "pdf") return "pdf-icon";
  if (filePreviewKind(name) === "video") return "video-icon";
  return "document-icon";
}

function formatFileSize(size) {
  if (typeof size !== "number" || !Number.isFinite(size) || size < 0)
    return "—";
  if (size < 1024) return `${size} B`;
  const unit = Math.min(Math.floor(Math.log(size) / Math.log(1024)), 3);
  return `${Number((size / 1024 ** unit).toFixed(1))} ${["B", "KB", "MB", "GB"][unit]}`;
}

function formatFileDate(value) {
  const date = new Date(value);
  return Number.isNaN(date.getTime())
    ? "—"
    : date.toLocaleDateString("en-GB", {
        day: "numeric",
        month: "long",
        year: "numeric",
      });
}

class FilesPreview extends Render {
  constructor({ entry, result, close, highlighter }) {
    const kind = result.kind;
    const content =
      kind === "text"
        ? new Render({
            tag: "pre",
            children: [new Render({ tag: "code", plainText: result.text })],
          })
        : kind === "image"
          ? new Render({ tag: "img", attrs: { alt: entry.name } })
          : kind === "pdf"
            ? new Render({
                tag: "iframe",
                attrs: { title: `Preview of ${entry.name}` },
              })
            : kind === "video"
              ? new Render({
                  tag: "video",
                  props: { controls: true },
                  attrs: { preload: "metadata" },
                })
              : new P({
                  plainText:
                    "Preview unavailable. Download the file to open it.",
                });
    super({
      tag: "dialog",
      classlist: ["files-preview"],
      attrs: { "aria-labelledby": "files-preview-title" },
      on: {
        cancel: (event) => {
          event.preventDefault();
          close();
        },
        click: (event) => {
          if (event.target !== event.currentTarget) return;
          const bounds = event.currentTarget.getBoundingClientRect();
          if (
            event.clientX < bounds.left ||
            event.clientX > bounds.right ||
            event.clientY < bounds.top ||
            event.clientY > bounds.bottom
          )
            close();
        },
      },
      children: [
        new Div({
          classlist: ["files-preview-header"],
          children: [
            new H2({ key: "files-preview-title", plainText: entry.name }),
            new A({
              plainText: "Download",
              classlist: ["files-preview-download"],
              attrs: { download: entry.name, "data-download": "" },
            }),
            new Button({
              classlist: ["files-preview-close"],
              plainText: "×",
              attrs: {
                type: "button",
                "aria-label": "Close preview",
                title: "Close preview",
                autofocus: true,
              },
              on: { click: close },
            }),
          ],
        }),
        new Div({
          classlist: ["files-preview-body"],
          children: [content],
        }),
      ],
      onmount: (element) => {
        const previousFocus = element.ownerDocument.activeElement;
        const page = element.ownerDocument.documentElement;
        const previousOverflow = page.style.overflow;
        this.addCleanup(() => {
          if (element.open) element.close();
          page.style.overflow = previousOverflow;
          if (previousFocus?.isConnected)
            previousFocus.focus({ preventScroll: true });
        });
        element.showModal();
        page.style.overflow = "hidden";
        const blobUrl = result.blob ? URL.createObjectURL(result.blob) : null;
        const url = blobUrl || result.url;
        const media = element.querySelector("img,iframe,video");
        if (media && url) media.src = url;
        const download = element.querySelector("[data-download]");
        if (url) download.href = url;
        else if (kind === "text") {
          const textUrl = URL.createObjectURL(
            new Blob([result.text], { type: "text/plain" }),
          );
          download.href = textUrl;
          this.addCleanup(() => URL.revokeObjectURL(textUrl));
        }
        let cleanup;
        try {
          cleanup =
            kind === "text"
              ? highlighter?.(element.querySelector("code"), entry)
              : undefined;
        } catch (error) {
          if (blobUrl) URL.revokeObjectURL(blobUrl);
          throw error;
        }
        return () => {
          try {
            if (typeof cleanup === "function") cleanup();
          } finally {
            try {
              if (kind === "video" && media) {
                try {
                  media.pause();
                } finally {
                  media.removeAttribute("src");
                  media.load();
                }
              } else media?.removeAttribute("src");
            } finally {
              if (blobUrl) URL.revokeObjectURL(blobUrl);
            }
          }
        };
      },
    });
  }
}

class FileBrowser extends Div {
  constructor({ api = defaultFilesApi(), path = "", highlighter } = {}) {
    const store = new State({
      persist: false,
      initial: {
        path: filesPath(path),
        entries: [],
        directories: {},
        query: "",
        selectedPath: "",
        sort: "name",
        ascending: true,
        loading: false,
        busy: false,
        error: "",
        preview: null,
        addOpen: false,
        folderOpen: false,
        uploadOpen: false,
      },
    });
    super({
      classlist: ["files-browser"],
      on: {
        dragover: (event) => {
          event.preventDefault();
          this.element.classList.add("dragover");
        },
        dragleave: (event) => {
          if (!this.element.contains(event.relatedTarget))
            this.element.classList.remove("dragover");
        },
        drop: (event) => {
          event.preventDefault();
          this.element.classList.remove("dragover");
          this.uploadFiles([...event.dataTransfer.files]);
        },
        keydown: (event) => {
          if (event.key === "Escape") {
            store.set("addOpen", false);
            if (!store.get("preview")) this.closePreview();
          }
        },
      },
    });
    this.api = api;
    this.state = store;
    this.highlighter = highlighter;
    this._loadId = this._previewId = this._mutationId = 0;
    const button = (plainText, click, classlist = [], attrs = {}) =>
      new Button({
        plainText,
        classlist,
        attrs: { type: "button", ...attrs },
        on: { click },
      });
    const pending = (config) => ({
      ...config,
      store,
      watch: ["busy", "loading"],
      attrs: {
        ...config.attrs,
        disabled: (state) => state.get("busy") || state.get("loading"),
      },
    });
    const hiddenPath = () =>
      new Input({
        store,
        watch: ["path"],
        options: { type: "hidden", name: "path" },
        props: { value: (state) => state.get("path") },
      });
    const openEditor = (kind) => {
      store.set("addOpen", false);
      store.set("folderOpen", kind === "folder");
      store.set("uploadOpen", kind === "upload");
    };
    const editor = (kind) =>
      new Form({
        classlist: ["files-editor"],
        attrs: {
          method: "post",
          action: kind === "folder" ? api.forms.newFolder : api.forms.upload,
          enctype: "multipart/form-data",
          "aria-label": kind === "folder" ? "Create folder" : "Upload file",
        },
        on: { submit: (event) => this._submit(event, kind) },
        children: [
          hiddenPath(),
          new Label({
            plainText: kind === "folder" ? "Folder name" : "Choose file",
            attrs: {
              for: kind === "folder" ? "files_folder_name" : "files_upload",
            },
          }),
          new Input({
            key: kind === "folder" ? "files_folder_name" : "files_upload",
            options: {
              type: kind === "folder" ? "text" : "file",
              name: kind === "folder" ? "name" : "file",
            },
            attrs: {
              required: true,
              placeholder: kind === "folder" ? "New folder" : null,
              autocomplete: "off",
            },
            onmount: (element) => element.focus(),
          }),
          new Button(
            pending({
              plainText: kind === "folder" ? "Create folder" : "Upload",
              classlist: ["button"],
              attrs: { type: "submit" },
            }),
          ),
          button(
            "Cancel",
            () =>
              store.set(kind === "folder" ? "folderOpen" : "uploadOpen", false),
            ["button", "secondary"],
          ),
        ],
      });
    const sortHeader = (label, field, className) =>
      new Th({
        classlist: [className],
        store,
        watch: ["sort", "ascending"],
        attrs: {
          scope: "col",
          "aria-sort": (state) =>
            state.get("sort") === field
              ? state.get("ascending")
                ? "ascending"
                : "descending"
              : "none",
        },
        children: [button(label, () => this.sort(field), ["sort-button"])],
      });
    const rows = new List({
      tag: "tbody",
      store,
      watch: ["entries", "sort", "ascending", "query"],
      items: (state) => this._sorted(state),
      key: (entry) => entry.path,
      render: (row) => this._row(row),
    });
    this.children = [
      new Render({
        tag: "header",
        classlist: ["files-header"],
        children: [
          new H1({ plainText: "Files" }),
          new Div({
            classlist: ["files-search"],
            children: [
              new Input({
                store,
                watch: ["query"],
                options: { type: "search" },
                attrs: {
                  "aria-label": "Search files and folders",
                  placeholder: "Search files and folders",
                },
                props: { value: (state) => state.get("query") },
                on: {
                  input: (event) => store.set("query", event.target.value),
                },
              }),
              new Render({
                tag: "span",
                classlist: ["search-icon"],
                attrs: { "aria-hidden": "true" },
              }),
            ],
          }),
        ],
      }),
      new Div({
        classlist: ["files-layout"],
        children: [
          new Render({
            tag: "aside",
            classlist: ["files-sidebar"],
            attrs: { "aria-label": "Folders and actions" },
            children: [
              new Div({
                classlist: ["files-add"],
                children: [
                  new Button({
                    plainText: "Add New",
                    classlist: ["button", "add-new"],
                    store,
                    watch: ["addOpen"],
                    attrs: {
                      type: "button",
                      "aria-expanded": (state) => state.get("addOpen"),
                      "aria-controls": "files-add-menu",
                    },
                    on: {
                      click: () => store.set("addOpen", !store.get("addOpen")),
                    },
                  }),
                  new Region({
                    store,
                    watch: ["addOpen"],
                    render: (state) =>
                      state.get("addOpen")
                        ? new Div({
                            key: "files-add-menu",
                            classlist: ["files-add-menu"],
                            children: [
                              button("New folder", () => openEditor("folder")),
                              button("Upload file", () => openEditor("upload")),
                            ],
                          })
                        : null,
                  }),
                ],
              }),
              new Render({
                tag: "nav",
                attrs: { "aria-label": "Folder navigation" },
                children: [
                  new A({
                    classlist: ["folder-root"],
                    store,
                    watch: ["path"],
                    classes: { "is-active": (state) => !state.get("path") },
                    attrs: {
                      href: "#/files/",
                      "aria-current": (state) =>
                        !state.get("path") ? "page" : null,
                    },
                    children: [
                      new Render({
                        tag: "span",
                        classlist: ["folder-chevron"],
                        plainText: "▾",
                        attrs: { "aria-hidden": "true" },
                      }),
                      this._icon(null, true),
                      new Render({ tag: "span", plainText: "My Files" }),
                    ],
                  }),
                  new Region({
                    store,
                    watch: ["directories", "path"],
                    render: (state) => this._folderTree(state),
                  }),
                ],
              }),
              new P({
                classlist: ["files-drop-hint"],
                plainText: "Drop files here to upload",
              }),
            ],
          }),
          new Render({
            tag: "section",
            classlist: ["files-panel"],
            attrs: { "aria-label": "File browser" },
            children: [
              new Div({
                classlist: ["files-panel-heading"],
                children: [
                  button(
                    "↻",
                    () => this.navigate(store.get("path")),
                    ["files-refresh"],
                    { "aria-label": "Refresh", title: "Refresh folder" },
                  ),
                  new Region({
                    store,
                    watch: ["path"],
                    render: (state) => this._breadcrumbs(state.get("path")),
                  }),
                ],
              }),
              new Region({
                store,
                watch: ["folderOpen"],
                render: (state) =>
                  state.get("folderOpen") ? editor("folder") : null,
              }),
              new Region({
                store,
                watch: ["uploadOpen"],
                render: (state) =>
                  state.get("uploadOpen") ? editor("upload") : null,
              }),
              new Region({
                store,
                watch: ["loading", "busy"],
                render: (state) =>
                  state.get("loading") || state.get("busy")
                    ? new P({
                        classlist: ["files-status"],
                        attrs: { role: "status" },
                        plainText: state.get("busy")
                          ? "Saving…"
                          : "Loading directory…",
                      })
                    : null,
              }),
              new Region({
                store,
                watch: ["error"],
                render: (state) =>
                  state.get("error")
                    ? new Div({
                        classlist: ["files-error"],
                        attrs: { role: "alert" },
                        children: [
                          new P({ plainText: state.get("error") }),
                          button(
                            "Retry",
                            () => this.navigate(store.get("path")),
                            ["button"],
                          ),
                        ],
                      })
                    : null,
              }),
              new Div({
                classlist: ["files-table-scroll"],
                children: [
                  new Table({
                    classlist: ["files-table"],
                    children: [
                      new Render({
                        tag: "caption",
                        classlist: ["visually-hidden"],
                        plainText: "Directory contents",
                      }),
                      new Thead({
                        children: [
                          new Tr({
                            children: [
                              sortHeader("Name", "name", "file-name-heading"),
                              sortHeader("Size", "size", "file-size-heading"),
                              sortHeader(
                                "Date",
                                "created",
                                "file-date-heading",
                              ),
                              new Th({
                                classlist: ["file-actions-heading"],
                                attrs: { scope: "col" },
                                children: [
                                  new Render({
                                    tag: "span",
                                    classlist: ["visually-hidden"],
                                    plainText: "Download",
                                  }),
                                ],
                              }),
                            ],
                          }),
                        ],
                      }),
                      rows,
                    ],
                  }),
                ],
              }),
              new Region({
                store,
                watch: ["entries", "query", "loading", "error"],
                render: (state) =>
                  !state.get("loading") &&
                  !state.get("error") &&
                  !this._sorted(state).length
                    ? new P({
                        classlist: ["files-empty"],
                        attrs: { role: "status" },
                        plainText: state.get("query")
                          ? "No files or folders match your search."
                          : "This folder is empty.",
                      })
                    : null,
              }),
              new Region({
                store,
                watch: ["preview"],
                render: (state) => {
                  const preview = state.get("preview");
                  return preview
                    ? new FilesPreview({
                        ...preview,
                        highlighter: this.highlighter,
                        close: () => this.closePreview(),
                      })
                    : null;
                },
              }),
            ],
          }),
        ],
      }),
    ];
    this.onmount = () => {
      this._active = true;
      this.navigate(store.get("path"));
      return () => {
        this._active = false;
        this._cancelLoads();
        store.set("preview", null);
        store.set("busy", false);
        store.set("loading", false);
      };
    };
  }

  _breadcrumbs(path) {
    const children = [
      new A({ plainText: "My Files", attrs: { href: "#/files/" } }),
    ];
    const segments = path.split("/").filter(Boolean);
    segments.forEach((name, index) => {
      children.push(
        new Render({
          tag: "span",
          classlist: ["breadcrumb-divider"],
          plainText: "/",
          attrs: { "aria-hidden": "true" },
        }),
      );
      children.push(
        new A({
          plainText: name,
          attrs: {
            href: `#/files/${filesEncodedPath(segments.slice(0, index + 1).join("/"))}`,
          },
        }),
      );
    });
    return new Render({
      tag: "nav",
      classlist: ["files-breadcrumbs"],
      attrs: { "aria-label": "Current directory" },
      children,
    });
  }

  _folderTree(store, parent = "") {
    const current = store.get("path");
    const folders = [...(store.get("directories")[parent] || [])];
    const segments = current.split("/").filter(Boolean);
    const depth = parent ? parent.split("/").length : 0;
    if (segments[depth]) {
      const path = segments.slice(0, depth + 1).join("/");
      if (!folders.some((folder) => folder.path === path))
        folders.push({ name: segments[depth], path });
    }
    folders.sort((a, b) =>
      a.name.localeCompare(b.name, undefined, {
        numeric: true,
        sensitivity: "base",
      }),
    );
    return new Render({
      tag: "ul",
      classlist: ["folder-tree"],
      children: folders.map(
        (folder) =>
          new Render({
            tag: "li",
            children: [
              new A({
                classlist: current === folder.path ? ["is-active"] : [],
                attrs: {
                  href: `#/files/${filesEncodedPath(folder.path)}`,
                  "aria-current": current === folder.path ? "page" : null,
                },
                children: [
                  this._icon(null, true),
                  new Render({ tag: "span", plainText: folder.name }),
                ],
              }),
              ...(current === folder.path ||
              current.startsWith(`${folder.path}/`)
                ? [this._folderTree(store, folder.path)]
                : []),
            ],
          }),
      ),
    });
  }

  _icon(store, folder = false) {
    const entry = () => store?.get("item");
    return new Render({
      tag: "span",
      store,
      watch: store ? ["item"] : [],
      attrs: {
        "aria-hidden": "true",
        class: () =>
          `file-icon ${folder || entry()?.kind === "folder" ? "folder-icon" : fileIconKind(entry()?.name)}`,
      },
      plainText: () =>
        folder || entry()?.kind === "folder"
          ? ""
          : fileIconLabel(entry()?.name),
    });
  }

  _sorted(store) {
    const field = store.get("sort");
    const direction = store.get("ascending") ? 1 : -1;
    const query = store.get("query", "").trim().toLocaleLowerCase();
    return store
      .get("entries")
      .filter((entry) => entry.name.toLocaleLowerCase().includes(query))
      .sort((a, b) => {
        if (a.kind !== b.kind) return a.kind === "folder" ? -1 : 1;
        if (field === "size")
          return ((a.size ?? 0) - (b.size ?? 0)) * direction;
        const aTime = field === "created" ? Date.parse(a.created) : NaN;
        const bTime = field === "created" ? Date.parse(b.created) : NaN;
        return (
          (Number.isFinite(aTime) && Number.isFinite(bTime)
            ? aTime - bTime
            : String(a[field]).localeCompare(String(b[field]), undefined, {
                numeric: true,
                sensitivity: "base",
              })) * direction
        );
      });
  }

  sort(field) {
    if (this.state.get("sort") === field)
      this.state.set("ascending", !this.state.get("ascending"));
    else {
      this.state.set("sort", field);
      this.state.set("ascending", true);
    }
  }

  _row(store) {
    const entry = () => store.get("item");
    return new Tr({
      store,
      watch: ["item", "selected"],
      classes: { "is-selected": (state) => state.get("selected", false) },
      attrs: { "data-file-path": () => entry().path },
      onmount: () => {
        const update = () =>
          store.set(
            "selected",
            this.state.get("selectedPath") === entry().path,
          );
        update();
        return this.state.subscribe("selectedPath", update);
      },
      children: [
        new Td({
          children: [
            new A({
              classlist: ["file-name"],
              attrs: {
                href: () =>
                  entry().kind === "folder"
                    ? `#/files/${filesEncodedPath(entry().path)}`
                    : this.api.fileUrl(entry()),
              },
              store,
              watch: ["item"],
              children: [
                this._icon(store),
                new Render({
                  tag: "span",
                  store,
                  watch: ["item"],
                  plainText: () => entry().name,
                }),
              ],
              on: {
                click: (event) => {
                  if (
                    entry().kind !== "folder" &&
                    !event.ctrlKey &&
                    !event.metaKey &&
                    !event.shiftKey &&
                    !event.altKey
                  ) {
                    event.preventDefault();
                    this.openPreview(entry());
                  }
                },
              },
            }),
          ],
        }),
        new Td({
          classlist: ["file-size"],
          store,
          watch: ["item"],
          plainText: () =>
            entry().kind === "folder" ? "" : formatFileSize(entry().size),
        }),
        new Td({
          classlist: ["file-date"],
          store,
          watch: ["item"],
          plainText: () => formatFileDate(entry().created),
        }),
        new Td({
          classlist: ["file-actions"],
          children: [
            new A({
              classlist: ["file-download"],
              store,
              watch: ["item"],
              plainText: "↓",
              attrs: {
                hidden: () => entry().kind === "folder",
                href: () =>
                  entry().kind === "folder" ? null : this.api.fileUrl(entry()),
                download: () => entry().name,
                "aria-label": () => `Download ${entry().name}`,
                title: "Download",
              },
            }),
          ],
        }),
      ],
    });
  }

  _cancelLoads() {
    this._loadId++;
    this._previewId++;
    this._mutationId++;
    this._directoryTask?.cleanups.splice(0).forEach((cleanup) => cleanup());
    this._directoryController?.abort();
    this._previewController?.abort();
    this._mutationController?.abort();
  }

  navigate(path = "", { signal } = {}) {
    path = filesPath(path);
    if (signal?.aborted) return Promise.resolve();
    const attachSignal = (task) => {
      if (!signal) return;
      const abort = () => task.controller.abort();
      signal.addEventListener("abort", abort, { once: true });
      task.cleanups.push(() => signal.removeEventListener("abort", abort));
    };
    if (
      this._directoryTask?.path === path &&
      !this._directoryTask.controller.signal.aborted
    ) {
      attachSignal(this._directoryTask);
      return this._directoryTask.promise;
    }
    this._cancelLoads();
    this.state.set("preview", null);
    this.state.set("busy", false);
    if (this.state.get("path") !== path) {
      this.state.set("entries", []);
      this.state.set("query", "");
    }
    this.state.set("selectedPath", "");
    this.state.set("path", path);
    this.state.set("error", "");
    this.state.set("loading", true);
    const id = this._loadId;
    const controller = (this._directoryController = new AbortController());
    const task = (this._directoryTask = {
      path,
      controller,
      cleanups: [],
      promise: null,
    });
    attachSignal(task);
    task.promise = (async () => {
      try {
        const entries = await this.api.list(path, {
          signal: controller.signal,
        });
        if (id !== this._loadId || controller.signal.aborted) return;
        this.state.set("entries", entries);
        this.state.set("directories", {
          ...this.state.get("directories"),
          [path]: entries.filter((entry) => entry.kind === "folder"),
        });
      } catch (error) {
        if (id === this._loadId && !controller.signal.aborted)
          this.state.set("error", error.message || String(error));
      } finally {
        task.cleanups.splice(0).forEach((cleanup) => cleanup());
        if (this._directoryTask === task) this._directoryTask = null;
        if (id === this._loadId) this.state.set("loading", false);
      }
    })();
    return task.promise;
  }

  closePreview() {
    this._previewId++;
    this._previewController?.abort();
    this.state.set("preview", null);
  }

  async openPreview(entry) {
    this.closePreview();
    this.state.set("selectedPath", entry.path);
    this.state.set("error", "");
    const id = this._previewId;
    const controller = (this._previewController = new AbortController());
    try {
      const result = await this.api.preview(entry, {
        signal: controller.signal,
      });
      if (id === this._previewId && !controller.signal.aborted)
        this.state.set("preview", { entry, result });
    } catch (error) {
      if (id === this._previewId && !controller.signal.aborted)
        this.state.set("error", error.message || String(error));
    }
  }

  async _submit(event, kind) {
    if (this.api.nativeForms) return;
    event.preventDefault();
    if (this.state.get("busy") || this.state.get("loading")) return;
    const form = event.currentTarget;
    const file =
      kind === "upload" ? form.elements.namedItem("file").files[0] : null;
    const name =
      kind === "folder" ? form.elements.namedItem("name").value.trim() : null;
    if (
      (kind === "upload" && !file) ||
      (kind === "folder" &&
        (!name || /[\\/]/.test(name) || [".", ".."].includes(name)))
    ) {
      this.state.set(
        "error",
        kind === "upload"
          ? "Choose a file first"
          : "Enter one folder name without slashes",
      );
      return;
    }
    if (kind === "upload") return this.uploadFiles([file]);
    return this._mutate(
      (path, signal) => this.api.createFolder(path, name, { signal }),
      () => form.reset(),
    );
  }

  uploadFiles(files) {
    if (!files.length) return;
    return this._mutate(
      async (path, signal) => {
        for (const file of files) {
          if (signal.aborted) return;
          await this.api.upload(path, file, { signal });
        }
      },
      () => this.state.set("uploadOpen", false),
    );
  }

  async _mutate(action, onSuccess) {
    if (this.state.get("busy") || this.state.get("loading")) return;
    this._mutationController?.abort();
    const controller = (this._mutationController = new AbortController());
    const id = ++this._mutationId;
    const path = this.state.get("path");
    this.state.set("busy", true);
    this.state.set("error", "");
    try {
      await action(path, controller.signal);
      if (id !== this._mutationId || controller.signal.aborted) return;
      onSuccess();
      await this.navigate(path);
    } catch (error) {
      if (id === this._mutationId && !controller.signal.aborted)
        this.state.set("error", error.message || String(error));
    } finally {
      if (id === this._mutationId) this.state.set("busy", false);
    }
  }
}
