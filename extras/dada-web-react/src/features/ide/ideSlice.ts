import { createSlice, PayloadAction } from "@reduxjs/toolkit";
import { RootState } from "../../app/store";

export type Cursor = { row: number; column: number };

export type CompilerState = {
  diagnostics: string;
  heaps: [string, string];
  output: string;
};
export interface IdeState {
  compilerState: CompilerState;
  cursor: Cursor;
  source: string;
}

const initialCompilerState = {
  diagnostics: "",
  heaps: ["", ""] as [string, string],
  output: ""
};

const initialSource = `async fn main() {
    print("
        I have forced myself to contradict myself
        in order to avoid conforming to my own taste.
          -- Marcel Duchamp
    ").await
  }`;
const initialState: IdeState = {
  compilerState: initialCompilerState,
  cursor: { row: 0, column: 0 },
  source: initialSource
};

export const ideSlice = createSlice({
  name: "compiler",
  initialState,
  // The `reducers` field lets us define reducers and generate associated actions
  reducers: {
    setCompilerState: (state, action: PayloadAction<CompilerState>) => {
      state.compilerState = action.payload;
    },
    setCursor: (state, action: PayloadAction<Cursor>) => {
      state.cursor = action.payload;
    },
    setSource: (state, action: PayloadAction<string>) => {
      state.source = action.payload;
    }
  }
});

export const { setCompilerState, setCursor, setSource } = ideSlice.actions;

// The function below is called a selector and allows us to select a value from
// the state.
export const selectCompilerState = (state: RootState) =>
  state.ide.compilerState;
export const selectCursor = (state: RootState) => state.ide.cursor;
export const selectSource = (state: RootState) => state.ide.source;

// For tests
export { initialCompilerState, initialSource };

export default ideSlice.reducer;
