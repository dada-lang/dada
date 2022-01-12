import init, { execute } from "./pkg/dada_web.js";
init()
    .then(async () => {
        var editor = ace.edit("editor");
        editor.setTheme("ace/theme/twilight");
        editor.session.on('change', async function (delta) {
            // delta.start, delta.end, delta.lines, delta.action
            try {
                let text = await execute(editor.getValue());
                var ansi_up = new AnsiUp;
                var html = ansi_up.ansi_to_html(text);
                var cdiv = document.getElementById("output");
                cdiv.innerHTML = html;
            } catch (e) { }
        });
        // editor.session.setMode("ace/mode/javascript");
    });
