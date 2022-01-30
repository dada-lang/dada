import { window as Window, ExtensionContext } from "vscode";
import { LanguageClient } from "vscode-languageclient/node";

let client: LanguageClient;

export function activate(_context: ExtensionContext) {
  const run = { command: "dada", args: ["ide"] };

  client = new LanguageClient(
    "dada-language-server",
    "Dada Language Server",
    {
      run,
      debug: {
        ...run,
        args: ["--log", "lsp,dada"].concat(run.args),
      },
    },
    {
      documentSelector: [{ scheme: "file", language: "dada" }],
      outputChannel: Window.createOutputChannel("Dada Language Server"),
    }
  );

  client.start();
}

export function deactivate(): Thenable<void> {
  if (!client) {
    return undefined;
  }
  return client.stop();
}
