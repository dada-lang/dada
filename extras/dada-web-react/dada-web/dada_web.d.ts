/* tslint:disable */
/* eslint-disable */
/**
*/
export function start(): void;
/**
* @returns {DadaCompiler}
*/
export function compiler(): DadaCompiler;
/**
*/
export class DadaCompiler {
  free(): void;
/**
* @param {string} source
* @returns {DadaCompiler}
*/
  with_source_text(source: string): DadaCompiler;
/**
* @param {number} line0
* @param {number} column0
* @returns {DadaCompiler}
*/
  with_breakpoint(line0: number, column0: number): DadaCompiler;
/**
* @returns {DadaCompiler}
*/
  without_breakpoint(): DadaCompiler;
/**
* @returns {Promise<DadaCompiler>}
*/
  execute(): Promise<DadaCompiler>;
/**
* @param {number} index
* @returns {DadaDiagnostic}
*/
  diagnostic(index: number): DadaDiagnostic;
/**
* @param {number} index
* @returns {DadaRange}
*/
  breakpoint_range(index: number): DadaRange;
/**
* @returns {string}
*/
  readonly diagnostics: string;
/**
* @returns {string}
*/
  readonly heap_after: string;
/**
* @returns {string}
*/
  readonly heap_before: string;
/**
* @returns {number}
*/
  readonly num_breakpoint_ranges: number;
/**
* @returns {number}
*/
  readonly num_diagnostics: number;
/**
* @returns {string}
*/
  readonly output: string;
}
/**
*/
export class DadaDiagnostic {
  free(): void;
/**
* @returns {DadaLabel}
*/
  readonly primary_label: DadaLabel;
/**
* @returns {string}
*/
  readonly severity: string;
}
/**
*/
export class DadaLabel {
  free(): void;
/**
*/
  end: DadaLineColumn;
/**
* @returns {string}
*/
  readonly message: string;
/**
*/
  start: DadaLineColumn;
}
/**
*/
export class DadaLineColumn {
  free(): void;
/**
*/
  column0: number;
/**
*/
  line0: number;
}
/**
*/
export class DadaRange {
  free(): void;
/**
*/
  end: DadaLineColumn;
/**
*/
  start: DadaLineColumn;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_dadarange_free: (a: number) => void;
  readonly __wbg_get_dadarange_start: (a: number) => number;
  readonly __wbg_set_dadarange_start: (a: number, b: number) => void;
  readonly __wbg_get_dadarange_end: (a: number) => number;
  readonly __wbg_set_dadarange_end: (a: number, b: number) => void;
  readonly __wbg_dadalinecolumn_free: (a: number) => void;
  readonly __wbg_get_dadalinecolumn_line0: (a: number) => number;
  readonly __wbg_set_dadalinecolumn_line0: (a: number, b: number) => void;
  readonly __wbg_get_dadalinecolumn_column0: (a: number) => number;
  readonly __wbg_set_dadalinecolumn_column0: (a: number, b: number) => void;
  readonly __wbg_dadadiagnostic_free: (a: number) => void;
  readonly __wbg_dadalabel_free: (a: number) => void;
  readonly __wbg_get_dadalabel_start: (a: number) => number;
  readonly __wbg_set_dadalabel_start: (a: number, b: number) => void;
  readonly __wbg_get_dadalabel_end: (a: number) => number;
  readonly __wbg_set_dadalabel_end: (a: number, b: number) => void;
  readonly dadadiagnostic_severity: (a: number, b: number) => void;
  readonly dadadiagnostic_primary_label: (a: number) => number;
  readonly dadalabel_message: (a: number, b: number) => void;
  readonly start: () => void;
  readonly __wbg_dadacompiler_free: (a: number) => void;
  readonly compiler: () => number;
  readonly dadacompiler_with_source_text: (a: number, b: number, c: number) => number;
  readonly dadacompiler_with_breakpoint: (a: number, b: number, c: number) => number;
  readonly dadacompiler_without_breakpoint: (a: number) => number;
  readonly dadacompiler_execute: (a: number) => number;
  readonly dadacompiler_num_diagnostics: (a: number) => number;
  readonly dadacompiler_diagnostic: (a: number, b: number) => number;
  readonly dadacompiler_num_breakpoint_ranges: (a: number) => number;
  readonly dadacompiler_breakpoint_range: (a: number, b: number) => number;
  readonly dadacompiler_diagnostics: (a: number, b: number) => void;
  readonly dadacompiler_output: (a: number, b: number) => void;
  readonly dadacompiler_heap_before: (a: number, b: number) => void;
  readonly dadacompiler_heap_after: (a: number, b: number) => void;
  readonly rust_psm_on_stack: (a: number, b: number, c: number, d: number) => void;
  readonly rust_psm_stack_direction: () => number;
  readonly rust_psm_stack_pointer: () => number;
  readonly rust_psm_replace_stack: (a: number, b: number, c: number) => void;
  readonly __wbindgen_malloc: (a: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number) => number;
  readonly __wbindgen_export_2: WebAssembly.Table;
  readonly _dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h6246e0110d1beb88: (a: number, b: number, c: number) => void;
  readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
  readonly __wbindgen_free: (a: number, b: number) => void;
  readonly __wbindgen_exn_store: (a: number) => void;
  readonly wasm_bindgen__convert__closures__invoke2_mut__h190f14940d20e675: (a: number, b: number, c: number, d: number) => void;
  readonly __wbindgen_start: () => void;
}

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {InitInput | Promise<InitInput>} module_or_path
*
* @returns {Promise<InitOutput>}
*/
export default function init (module_or_path?: InitInput | Promise<InitInput>): Promise<InitOutput>;
