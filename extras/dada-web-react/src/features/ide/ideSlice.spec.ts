import ideReducer, { IdeState, initialSource, setSource } from "./ideSlice";

describe("ide reducer", () => {
  const initialState: IdeState = {
    compilerState: { diagnostics: "", heaps: ["", ""], output: "" },
    cursor: { row: -1, column: -1 },
    source: ""
  };
  it("should handle initial state", () => {
    expect(ideReducer(undefined, { type: "unknown" })).toEqual({
      source: initialSource
    });
  });

  it("should handle setSource", () => {
    const actual = ideReducer(initialState, setSource("print(hello world)"));
    expect(actual.source).toEqual("print(hello world)");
  });
});
