import { useEffect, useState } from "react";
import Col from "react-bootstrap/Col";
import Row from "react-bootstrap/Row";

import dadaWeb, { compiler } from "dada-web";
import type { DadaCompiler, InitOutput } from "dada-web";

import Editor from "./editor";
import Output from "./output";

/**
 * Wrapper on the DadaCompiler to have a stable reference.
 */
class DCW {
  dada: DadaCompiler;
  constructor() {
    this.dada = compiler();
  }

  setSourceText(text: string) {
    this.dada = this.dada.with_source_text(text);
  }

  setBreakpoint(row: number, column: number) {
    if (row && column) this.dada = this.dada.with_breakpoint(row, column);
    else this.dada = this.dada.without_breakpoint();
  }

  async execute() {
    this.dada = await this.dada.execute();
  }

  get output() {
    return this.dada.output;
  }

  get diagnostics() {
    return this.dada.diagnostics;
  }

  get num_diagnostics() {
    return this.dada.num_diagnostics;
  }

  diagnostic(index: number) {
    return this.dada.diagnostic(index);
  }

  get num_breakpoint_ranges() {
    return this.dada.num_breakpoint_ranges;
  }

  breakpoint_range(index: number) {
    return this.dada.breakpoint_range(index);
  }

  get heaps() {
    return [this.dada.heap_before, this.dada.heap_after];
  }
}

export type Cursor = { row: number; column: number };

function Ide(props: { sourceText: string }) {
  const [_module, setModule] = useState<InitOutput | null>(null);
  const [dada, setDada] = useState<DCW | null>(null);

  // First pass: we have to initialize the webassembly and "DCW"
  // instance.
  useEffect(() => {
    async function initModule() {
      // Load the web assembly module
      const c = await dadaWeb();
      setModule(c);
      setDada(new DCW());
    }
    initModule();
  }, []);

  // Second pass: now that `dada != null`, we can do the rest.
  const [cursor, setCursor] = useState<Cursor>({ row: 0, column: 0 });
  const [source, setSource] = useState<string>(props.sourceText);
  const [diagnostics, setDiagnostics] = useState<string>("");
  const [output, setOutput] = useState<string>("");
  const [heaps, setHeaps] = useState<[string, string]>(["", ""]);
  useEffect(() => {
    async function updateCompiler() {
      if (!dada) return;
      dada.setSourceText(source);
      dada.setBreakpoint(cursor.row, cursor.column);
      await dada.execute();
      setOutput(dada.output);
      setHeaps([dada.heaps[0], dada.heaps[1]]);
      setDiagnostics(dada.diagnostics);
    }
    updateCompiler();
  }, [cursor, dada, source]);

  return (
    <Row>
      <Col>
        <Editor source={source} onCursorChange={setCursor} onSourceChange={setSource} />
      </Col>
      <Col>
        <Output output={output} heaps={heaps} />
      </Col>
    </Row>
  );
}

export default Ide;
