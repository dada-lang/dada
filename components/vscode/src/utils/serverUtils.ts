import * as path from 'path';
import * as os from 'os';
import * as fs from 'fs';
import * as vscode from 'vscode';
import * as child_process from 'child_process';
import { ExecutableOptions } from 'vscode-languageclient/node';

export function getServerPath(context: vscode.ExtensionContext): string {
  console.log('Dada LSP: Extension path is', context.extensionPath);

  // 1. Check user configuration first
  const config = vscode.workspace.getConfiguration('dada');
  const configuredPath = config.get<string>('serverPath');
  console.log('Dada LSP: Configured server path from settings:', configuredPath);
  if (configuredPath && fs.existsSync(configuredPath)) {
    console.log('Dada LSP: Using configured server path:', configuredPath);
    return configuredPath;
  }

  // 2. Try to use bundled binary based on platform
  const platform = os.platform();
  const arch = os.arch();

  let binaryName = 'dada-lsp-server';
  if (platform === 'win32') {
    binaryName += '.exe';
  }

  console.log('Dada LSP: extension path', context.extensionPath);

  const platformDir = `${platform === 'win32' ? 'win32' : platform === 'darwin' ? 'darwin' : 'linux'}-${arch === 'x64' ? 'x64' : arch === 'arm64' ? 'arm64' : 'x64'}`;
  const bundledPath = path.join(context.extensionPath, 'bin', platformDir, binaryName);
  console.log('Dada LSP: Calculated bundled server path:', bundledPath);

  if (fs.existsSync(bundledPath)) {
    console.log('Dada LSP: Bundled server binary exists');
    // Make sure the binary is executable on Unix systems
    if (platform !== 'win32') {
      try {
        fs.chmodSync(bundledPath, '755');
      } catch (e) {
        console.error('Failed to make server binary executable:', e);
      }
    }
    console.log('Dada LSP: Using bundled server path:', bundledPath);
    return bundledPath;
  } else {
    console.log('Dada LSP: Bundled server binary does not exist at path:', bundledPath);
  }

  // 3. Development fallback: try to use cargo run
  try {
    // Check if we're in a development environment with Cargo available
    child_process.execSync('cargo --version', { stdio: 'ignore' });

    // Return a special marker that tells the extension to use cargo run
    console.log('Dada LSP: Using cargo run for development');
    return 'USE_CARGO_RUN';
  } catch (e) {
    console.log('Dada LSP: Cargo not available for development fallback');
    // Cargo not available
  }

  // 4. If all else fails, assume it's in PATH
  console.log('Dada LSP: Falling back to binary in PATH:', binaryName);
  return binaryName;
}

export function getServerOptions(context: vscode.ExtensionContext,
  serverPath: string): { command: string, args: string[], options: ExecutableOptions } {
  if (serverPath === 'USE_CARGO_RUN') {
    return {
      command: 'cargo',
      args: ['run', '-p', 'dada-lsp-server', '-q', '--'],
      options: {
        cwd: context.extensionPath,
      }
    };
  }

  return {
    command: serverPath,
    args: [],
    options: {},
  };
}
