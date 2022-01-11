import init, { execute } from "./pkg/dada_web.js";
init()
    .then(async () => {
        var editor = ace.edit("editor");
        editor.setTheme("ace/theme/twilight");
        editor.session.on('change', async function (delta) {
            // delta.start, delta.end, delta.lines, delta.action
            console.log(`text is ${editor.getValue()}`);
            console.log(await execute(editor.getValue()));
        });
        // editor.session.setMode("ace/mode/javascript");
    });