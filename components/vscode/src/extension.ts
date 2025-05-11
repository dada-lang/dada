import * as vscode from 'vscode';
import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
  TransportKind
} from 'vscode-languageclient/node';
import { getServerPath, getServerOptions } from './utils/serverUtils';

let client: LanguageClient | undefined;

export function activate(context: vscode.ExtensionContext) {
  // Get server path using our utility
  const serverPath = getServerPath(context);
  const { command, args } = getServerOptions(serverPath);
  
  // Create output channel for logging
  const outputChannel = vscode.window.createOutputChannel('Dada Language Server');
  outputChannel.appendLine(`Using server: ${command} ${args.join(' ')}`);
  
  // Options to control the language client
  const clientOptions: LanguageClientOptions = {
    documentSelector: [{ scheme: 'file', language: 'dada' }],
    synchronize: {
      fileEvents: vscode.workspace.createFileSystemWatcher('**/*.dada')
    },
    outputChannel
  };

  // Create the server options
  const serverOptions: ServerOptions = {
    run: { command, args, transport: TransportKind.stdio },
    debug: { command, args, transport: TransportKind.stdio }
  };

  // Create and start the client
  client = new LanguageClient(
    'dada',
    'Dada Language Server',
    serverOptions,
    clientOptions
  );

  // Start the client
  client.start().catch(error => {
    vscode.window.showErrorMessage(
      `Failed to start Dada language server: ${error.message}. ` +
      'Please check the Dada Language Server output channel for details.'
    );
  });

  // Register commands
  context.subscriptions.push(
    vscode.commands.registerCommand('dada.restartServer', async () => {
      if (client) {
        await client.stop();
        client.start();
      }
    })
  );
}

export function deactivate(): Thenable<void> | undefined {
  if (!client) {
    return undefined;
  }
  return client.stop();
}
