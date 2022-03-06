import React from "react";
import AceEditor from "react-ace";

import "ace-builds/src-noconflict/mode-javascript";
import "ace-builds/src-noconflict/theme-github";
// import "ace-builds/src-noconflict/theme-twilight";

import { useAppSelector, useAppDispatch } from "../../app/hooks";
import type { AppDispatch } from "../../app/store";

import { selectSource, setCursor, setSource } from "./ideSlice";
import type { Cursor } from "./ideSlice";

type Selection = {
  cursor: Cursor;
};

function onChange(newValue: string, dispatch: AppDispatch) {
  dispatch(setSource(newValue));
}

function onCursorChange(selection: Selection, dispatch: AppDispatch) {
  dispatch(
    setCursor({ row: selection.cursor.row, column: selection.cursor.column })
  );
}

function Editor() {
  const source = useAppSelector(selectSource);
  const dispatch = useAppDispatch();

  return (
    <AceEditor
      editorProps={{ $blockScrolling: true }}
      fontSize={18}
      name="dada-editor"
      onChange={(v) => onChange(v, dispatch)}
      onCursorChange={(selection) => onCursorChange(selection, dispatch)}
      value={source}
      theme="github"
    />
  );
}

export default Editor;
