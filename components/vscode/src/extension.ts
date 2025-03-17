import * as vscode from 'vscode';
import * as wasmExtension from './wasm-extension';
import * as nativeExtension from './native-extension';

const USE_WASM = false;

export function activate(context: vscode.ExtensionContext) {
	if (USE_WASM) {
		wasmExtension.activate(context);
	} else {
		nativeExtension.activate(context);
	}
}

export function deactivate(): Thenable<void> | undefined {
	if (USE_WASM) {
		return wasmExtension.deactivate();
	} else {
		return nativeExtension.deactivate();
	}
}
