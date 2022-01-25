import init, { compiler } from "./pkg/dada_web.js";

const workerURL = 'ace/viz.render.js';
let viz = new Viz({ workerURL });

class Queue {
    constructor() {
        this.active = 0;
        this.queue = [];
    }

    // Submit a workFunction to the queue -- when called, this
    // should return a promise. It will be called once the
    // active worker has gotten around to it.
    submit(workFunction) {
        this.queue.push(workFunction);

        if (!this.active) {
            this.active = 1;
            this.doWork();
        }
    }

    async doWork() {
        while (this.queue.length != 0) {
            let workFunction = this.queue.shift();
            let promise = workFunction();
            await promise;
        }
        this.active = 0;
    }
}

// Wrapper around the raw wasm-bindgen API.
// Because you can't make async functions with `&mut self`,
// we tend to pass ownership of the dada compiler back and
// forth, which then requires a wrapper to track it.
class DadaCompiler {
    constructor() {
        this.dada = compiler();
    }

    setSourceText(text) {
        this.dada = this.dada.with_source_text(text)
    }

    setBreakpoint(cursor) {
        if (cursor) {
            this.dada = this.dada.with_breakpoint(cursor.row, cursor.column);
        } else {
            this.dada = this.dada.without_breakpoint();
        }
    }

    async execute() {
        this.dada = await this.dada.execute();
    }

    get output() {
        return this.dada.output;
    }

    get diagnostics() {
        return this.dada.diagnostics;
    }

    get num_diagnostics() {
        return this.dada.num_diagnostics;
    }

    diagnostic(i) {
        return this.dada.diagnostic(i);
    }

    get heap() {
        return this.dada.heap;
    }
}

init()
    .then(async () => {
        var dada = new DadaCompiler();
        let queue = new Queue();

        var editor = ace.edit("editor");
        editor.setTheme("ace/theme/twilight");
        editor.setOptions({
            fontSize: "18px"
        });
        editor.session.on('change', function (delta) {
            // delta.start, delta.end, delta.lines, delta.action
            setStatusMessage("");
            let text = editor.getValue();
            queue.submit(async function () {
                await updateOutput(dada, editor, text, null);
            });
        });
        editor.session.getSelection().on('changeCursor', function (arg) {
            let text = editor.getValue();
            let cursor = editor.session.getSelection().getCursor();
            console.log("changeCursor", cursor.row, cursor.column);
            queue.submit(async function () {
                await updateOutput(dada, editor, text, cursor);
            });
        });
        // editor.session.setMode("ace/mode/javascript");

        updateFromQueryString(editor);

        var button = document.getElementById("shareButton");
        button.onclick = async function (event) {
            await copyClipboardUrl(editor);
        }

        let text = editor.getValue();
        queue.submit(async function () {
            await updateOutput(dada, editor, text, null);
        });
    });

// Check if the user accessed `playground?code=foo` and, if so,
// update the code sample from `code`.
function updateFromQueryString(editor) {
    let params = new URLSearchParams(document.location.search);
    let code = params.get("code"); // is the string "Jonathan"
    if (code == null) {
        return;
    }

    editor.setValue(code);
}

function setStatusMessage(text) {
    var span = document.getElementById("statusSpan");
    span.innerText = text;
}

async function copyClipboardUrl(editor) {
    // get URL of the playground, and clear existing parameters
    var playgroundUrl = new URL(document.location.href);
    playgroundUrl.search = "?"; // clear existing parameters

    // set the ?code=xxx parameter
    let code = editor.getValue();
    playgroundUrl.searchParams.set("code", code);

    // minify
    let minifiedUrl = await minify(playgroundUrl);
    await navigator.clipboard.writeText(minifiedUrl.href);

    setStatusMessage("url copied to clipboard");
}

// Use the is.gd service to minify a URL.
// If the request fails, returns the unminified URL.
async function minify(url) {
    // Use the is.gd 
    // ?format=simple&url=www.example.com

    let isGdUrl = new URL("https://is.gd/create.php");
    isGdUrl.searchParams.set("format", "simple");
    isGdUrl.searchParams.set("url", url.href);

    try {
        let response = await fetch(isGdUrl);
        let text = await response.text();
        return new URL(text);
    } catch (e) {
        return url;
    }
}

async function updateOutput(dada, editor, text, cursor) {
    console.log("updateOutput", dada, text, cursor);

    try {
        dada.setSourceText(text);
        dada.setBreakpoint(cursor);

        await dada.execute();

        // Append diagnostics (if any) to stdout.
        let diagnostics = dada.diagnostics;
        let output = (diagnostics == "" ? dada.output : diagnostics + "\n" + dada.output);

        // Create Ace annotations for the errors.
        let annotations = [];
        for (let i = 0; i < dada.num_diagnostics; i++) {
            console.log("diagnostic ", i);
            let diagnostic = dada.diagnostic(i);
            console.log("severity", diagnostic.severity);
            let primary_label = diagnostic.primary_label;
            console.log("primary label", primary_label.start.line0, primary_label.start.column0, primary_label.message);
            let severity = diagnostic.severity == "error" ? "error"
                : diagnostic.severity == "warning" ? "warning" :
                    "information";
            annotations.push({
                row: primary_label.start.line0,
                column: primary_label.start.column0,
                text: primary_label.message,
                type: severity,
            });
        }
        console.log("annoattions", annotations);
        editor.getSession().setAnnotations(annotations);

        // Take the console output, convert it to html, and put it in the
        // output area.
        var html = (new AnsiUp).ansi_to_html(output);
        document.getElementById("output").innerHTML = html;

        // Clear any previous heap capture nodes.
        var gdiv = document.getElementById("graph");
        while (gdiv.firstChild != null) {
            gdiv.removeChild(gdiv.firstChild);
        }

        // If the result included any heapcapture, it will be encoded
        // as a graphviz string. Use viz.js to convert that to SVG,
        // and then add the SVG nodes there.
        let heap = dada.heap;
        if (heap != "") {
            try {
                let element = await viz.renderSVGElement(heap);
                gdiv.appendChild(element);
            } catch (error) {
                viz = new Viz({ workerURL });
                console.log("rendering error", error);
            }
        }
    } catch (e) { }
}