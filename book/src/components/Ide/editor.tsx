import React, { PropsWithChildren } from "react";
import AceEditor from "react-ace";
import { useColorMode } from "@docusaurus/theme-common";

import "ace-builds/src-noconflict/mode-rust";
import "ace-builds/src-noconflict/theme-github";
import "ace-builds/src-noconflict/theme-twilight";

import type { Cursor } from ".";

type OutputProps = {
    source: string;
    onCursorChange: (c: Cursor) => void;
    onSourceChange: (s: string) => void;
};

function Editor(props: PropsWithChildren<OutputProps>) {
    const { colorMode } = useColorMode();
    return (
        <AceEditor
            mode="rust"
            editorProps={{ $blockScrolling: true }}
            fontSize={18}
            width="100%"
            height="82vh"
            name="dada-editor"
            onChange={(v) => props.onSourceChange(v)}
            onCursorChange={(selection) =>
                props.onCursorChange({
                    row: selection.cursor.row,
                    column: selection.cursor.column,
                })
            }
            value={props.source}
            theme={colorMode === "dark" ? "twilight" : "github"}
        />
    );
}

export default Editor;
