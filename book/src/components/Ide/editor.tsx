import React, { PropsWithChildren } from "react";
import AceEditor from "react-ace";

import "ace-builds/src-noconflict/mode-javascript";
import "ace-builds/src-noconflict/theme-github";
// import "ace-builds/src-noconflict/theme-twilight";

import type { Cursor } from ".";

type OutputProps = {
  source: string;
  onCursorChange: (c: Cursor) => void;
  onSourceChange: (s: string) => void;
};

function Editor(props: PropsWithChildren<OutputProps>) {
  return (
    <AceEditor
      editorProps={{ $blockScrolling: true }}
      fontSize={18}
      width="100%"
      height="82vh"
      name="dada-editor"
      onChange={(v) => props.onSourceChange(v)}
      onCursorChange={(selection) => props.onCursorChange({ row: selection.cursor.row, column: selection.cursor.column })}
      value={props.source}
      theme="github"
    />
  );
}

export default Editor;
