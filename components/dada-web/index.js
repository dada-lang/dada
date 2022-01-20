import init, { execute, execute_until } from "./pkg/dada_web.js";
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

        await updateOutput(editor);
    });

async function updateOutput(editor, cursor) {
    try {
        let result;
        if (cursor == null) {
            result = await execute(editor.getValue());
        } else {
            result = await execute_until(editor.getValue(), cursor.row + 1, cursor.column + 1);
        }
        console.log("executed until cursor: ", JSON.stringify(cursor));
        var ansi_up = new AnsiUp;
        var html = ansi_up.ansi_to_html(result.fullOutput);
        var cdiv = document.getElementById("output");
        cdiv.innerHTML = html;
    } catch (e) { }
}