import * as path from 'path';
import * as os from 'os';
import * as fs from 'fs';
import * as vscode from 'vscode';
import * as child_process from 'child_process';

export function getServerPath(context: vscode.ExtensionContext): string {
  // 1. Check user configuration first
  const config = vscode.workspace.getConfiguration('dada');
  const configuredPath = config.get<string>('serverPath');
  if (configuredPath && fs.existsSync(configuredPath)) {
    return configuredPath;
  }

  // 2. Try to use bundled binary based on platform
  const platform = os.platform();
  const arch = os.arch();
  
  let binaryName = 'dada-lsp-server';
  if (platform === 'win32') {
    binaryName += '.exe';
  }
  
  const platformDir = `${platform === 'win32' ? 'win32' : platform === 'darwin' ? 'darwin' : 'linux'}-${arch === 'x64' ? 'x64' : arch === 'arm64' ? 'arm64' : 'x64'}`;
  const bundledPath = path.join(context.extensionPath, 'bin', platformDir, binaryName);
  
  if (fs.existsSync(bundledPath)) {
    // Make sure the binary is executable on Unix systems
    if (platform !== 'win32') {
      try {
        fs.chmodSync(bundledPath, '755');
      } catch (e) {
        console.error('Failed to make server binary executable:', e);
      }
    }
    return bundledPath;
  }
  
  // 3. Development fallback: try to use cargo run
  try {
    // Check if we're in a development environment with Cargo available
    child_process.execSync('cargo --version', { stdio: 'ignore' });
    
    // Return a special marker that tells the extension to use cargo run
    return 'USE_CARGO_RUN';
  } catch (e) {
    // Cargo not available
  }
  
  // 4. If all else fails, assume it's in PATH
  return binaryName;
}

export function getServerOptions(serverPath: string): {command: string, args: string[]} {
  if (serverPath === 'USE_CARGO_RUN') {
    return {
      command: 'cargo',
      args: ['run', '-p', 'dada-lsp-server', '--']
    };
  }
  
  return {
    command: serverPath,
    args: []
  };
}
