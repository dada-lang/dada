import init, { compiler } from "./pkg/dada_web.js";

const workerURL = 'ace/viz.render.js';
let viz = new Viz({ workerURL });

// Wrapper around the raw wasm-bdingen API.
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

    get heap() {
        return this.dada.heap;
    }
}

init()
    .then(async () => {
        var dada = new DadaCompiler();

        var editor = ace.edit("editor");
        editor.setTheme("ace/theme/twilight");
        editor.setOptions({
            fontSize: "18px"
        });
        editor.session.on('change', async function (delta) {
            // delta.start, delta.end, delta.lines, delta.action
            setStatusMessage("");
            await updateOutput(dada, editor, null);
        });
        editor.session.getSelection().on('changeCursor', async function (arg) {
            let cursor = editor.session.getSelection().getCursor();
            console.log("changeCursor", cursor.row, cursor.column);
            await updateOutput(dada, editor, cursor);
        });
        // editor.session.setMode("ace/mode/javascript");

        updateFromQueryString(editor);

        var button = document.getElementById("shareButton");
        button.onclick = async function (event) {
            await copyClipboardUrl(editor);
        }

        await updateOutput(dada, editor, null);
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

async function updateOutput(dada, editor, cursor) {
    console.log("updateOutput: ", dada, cursor);
    try {
        dada.setSourceText(editor.getValue());
        dada.setBreakpoint(cursor);

        console.log("executing until cursor: ", JSON.stringify(cursor));
        await dada.execute();
        console.log("executed");
        console.log("diagnostics:", dada.diagnostics);
        console.log("output:", dada.output);

        // Append diagnostics (if any) to stdout.
        let diagnostics = dada.diagnostics;
        let output = (diagnostics == "" ? dada.output : diagnostics + "\n" + dada.output);

        // Take the console output, convert it to html, and put it in the
        // output area.
        var html = (new AnsiUp).ansi_to_html(output);
        console.log("html:", html);
        document.getElementById("output").innerHTML = html;

        // If the result included any heapcapture, it will be encoded
        // as a graphviz string. Use viz.js to convert that to SVG,
        // clear out the existing contents from `#graph`,
        // and then add the SVG nodes there.
        let heap = dada.heap;
        if (heap != "") {
            try {
                let element = await viz.renderSVGElement(heap);
                var gdiv = document.getElementById("graph");
                while (gdiv.firstChild != null) {
                    gdiv.removeChild(gdiv.firstChild);
                }
                gdiv.appendChild(element);
            } catch (error) {
                viz = new Viz({ workerURL });
                console.log("rendering error", error);
            }
        }
    } catch (e) { }
}