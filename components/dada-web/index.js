import init, { execute, execute_until } from "./pkg/dada_web.js";

const workerURL = 'ace/viz.render.js';
let viz = new Viz({ workerURL });

init()
    .then(async () => {
        var editor = ace.edit("editor");
        editor.setTheme("ace/theme/twilight");
        editor.setOptions({
            fontSize: "18px"
        });
        editor.session.on('change', async function (delta) {
            // delta.start, delta.end, delta.lines, delta.action
            await updateOutput(editor, null);
        });
        editor.session.getSelection().on('changeCursor', async function (arg) {
            let cursor = editor.session.getSelection().getCursor();
            console.log("changeCursor", cursor.row, cursor.column);
            await updateOutput(editor, cursor);
        });
        // editor.session.setMode("ace/mode/javascript");

        updateFromQueryString(editor);

        await updateOutput(editor, null);
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

async function updateOutput(editor, cursor) {
    try {
        let result;
        if (cursor == null) {
            result = await execute(editor.getValue());
        } else {
            result = await execute_until(editor.getValue(), cursor.row, cursor.column);
        }

        console.log("executed until cursor: ", JSON.stringify(cursor));
        var ansi_up = new AnsiUp;
        var html = ansi_up.ansi_to_html(result.fullOutput);
        var cdiv = document.getElementById("output");
        cdiv.innerHTML = html;

        let heapCapture = result.heapCapture;
        if (heapCapture != "") {
            try {
                let element = await viz.renderSVGElement(heapCapture);
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