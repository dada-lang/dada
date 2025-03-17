import * as path from 'path';
import * as vscode from 'vscode';
import { LanguageClient, LanguageClientOptions, ServerOptions, TransportKind } from 'vscode-languageclient/node';

let client: LanguageClient;

export function activate(context: vscode.ExtensionContext) {
	// Path to the language server executable
	const serverModule = context.asAbsolutePath("../../target/debug/dada-lsp-server");

	// Server options
	const serverOptions: ServerOptions = {
		run: { command: serverModule, transport: TransportKind.stdio },
		debug: { command: serverModule, transport: TransportKind.stdio }
	};

	// Client options
	const clientOptions: LanguageClientOptions = {
		documentSelector: [{ scheme: 'file', language: 'dada' }],
		synchronize: {
			fileEvents: vscode.workspace.createFileSystemWatcher('**/*.dada')
		}
	};

	// Create the language client and start the client.
	client = new LanguageClient(
		'dadaLanguageServer',
		'Dada Language Server',
		serverOptions,
		clientOptions
	);

	// Start the client. This will also launch the server
	client.start();
}

export function deactivate(): Thenable<void> | undefined {
	if (!client) {
		return undefined;
	}
	return client.stop();
}