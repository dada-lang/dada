# Dada Language Support for Visual Studio Code

This extension provides language support for the Dada programming language.

## Features

- Syntax highlighting for `.dada` files
- Language server integration providing:
  - Error checking and diagnostics
  - Hover information
  - Go to definition

## Requirements

The extension includes pre-compiled binaries of the Dada language server for common platforms (Windows, macOS, and Linux). If a binary for your platform is not included, the extension will attempt to:

1. Use a custom path specified in the settings
2. Build and run the server using Cargo (requires Rust to be installed)
3. Find the server in your PATH

## Extension Settings

This extension contributes the following settings:

* `dada.serverPath`: Path to the Dada language server executable (optional)
* `dada.trace.server`: Traces the communication between VS Code and the Dada language server

## Commands

* `Dada: Restart Language Server`: Restarts the language server if it encounters issues

## Development

### Building the Extension

1. Install dependencies:
   ```
   npm install
   ```

2. Compile TypeScript:
   ```
   npm run compile
   ```

3. Package the extension:
   ```
   npx vsce package
   ```

### Adding Language Server Binaries

Pre-compiled binaries for the language server should be placed in the appropriate platform directory under `bin/`:

- `bin/darwin-x64/` - macOS Intel
- `bin/darwin-arm64/` - macOS Apple Silicon
- `bin/linux-x64/` - Linux
- `bin/win32-x64/` - Windows

During development, if no binary is found, the extension will attempt to use `cargo run -p dada-lsp-server --` to build and run the server on-demand.

## Known Issues

This is an early version of the extension and may have some limitations.

## Release Notes

### 0.1.0

Initial release of the Dada language extension.
