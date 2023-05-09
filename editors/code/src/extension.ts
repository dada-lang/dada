import { WorkspaceConfiguration, workspace } from "vscode";

import {
  Executable,
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
} from "vscode-languageclient/node";

let client: LanguageClient | undefined = undefined;

export async function activate() {
  const config = workspace.getConfiguration("dadaLanguageServer");

  const run: Executable = {
    command: getExecutable(config),
    args: ["ide"],
  };

  const serverOptions: ServerOptions = {
    run,
    debug: run,
  };

  const clientOptions: LanguageClientOptions = {
    documentSelector: [{ scheme: "file", language: "dada" }],
  };

  client = new LanguageClient(
    "dadaLanguageServer",
    "Dada Language Server",
    serverOptions,
    clientOptions
  );

  await client.start();
}

export function deactivate(): Thenable<void> | undefined {
  if (!client) {
    return undefined;
  }
  return client.stop();
}

function getExecutable(config: WorkspaceConfiguration): string {
  const explicitPath =
    process.env.__DADA_LSP_SERVER_DEBUG ??
    config.get<string | null>("compiler.executablePath");
  return explicitPath ?? "dada";
}
