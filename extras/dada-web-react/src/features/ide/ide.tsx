import React, { useEffect, useState } from "react";
import Col from "react-bootstrap/Col";
import Row from "react-bootstrap/Row";

import dadaWeb, { compiler } from "dada-web";
import type { DadaCompiler, InitOutput } from "dada-web";

import { useAppSelector, useAppDispatch } from "../../app/hooks";

import Editor from "./editor";
import { selectCursor, selectSource, setCompilerState } from "./ideSlice";
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

function Ide() {
  const dispatch = useAppDispatch();
  const [_module, setModule] = useState<InitOutput | null>(null);
  const [dada, setDada] = useState<DCW | null>(null);
  useEffect(() => {
    async function initModule() {
      const c = await dadaWeb();
      setModule(c);
      setDada(new DCW());
    }
    initModule();
  }, []);

  const source = useAppSelector(selectSource);
  const cursor = useAppSelector(selectCursor);
  useEffect(() => {
    async function updateCompiler() {
      if (!dada) return;
      dada.setSourceText(source);
      dada.setBreakpoint(cursor.row, cursor.column);
      await dada.execute();
      dispatch(
        setCompilerState({
          diagnostics: dada.diagnostics,
          heaps: [dada.heaps[0], dada.heaps[1]],
          output: dada.output
        })
      );
    }
    updateCompiler();
  }, [cursor, dada, dispatch, source]);

  return (
    <Row>
      <Col>
        <h1>Dada Source</h1>
        <Editor />
      </Col>
      <Col>
        <h1>Compiler Output</h1>
        <Output />
      </Col>
    </Row>
  );
}

export default Ide;
