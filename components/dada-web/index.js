import init, { execute } from "./pkg/dada_web.js";
init()
    .then(async () => {
        var editor = ace.edit("editor");
        editor.setTheme("ace/theme/twilight");
        editor.setOptions({
            fontSize: "18px"
        });
        editor.session.on('change', async function (delta) {
            // delta.start, delta.end, delta.lines, delta.action
            await updateOutput(editor);
        });
        // editor.session.setMode("ace/mode/javascript");

        await updateOutput(editor);
    });

async function updateOutput(editor) {
    try {
        let text = await execute(editor.getValue());
        var ansi_up = new AnsiUp;
        var html = ansi_up.ansi_to_html(text);
        var cdiv = document.getElementById("output");
        cdiv.innerHTML = html;
    } catch (e) { }
}
