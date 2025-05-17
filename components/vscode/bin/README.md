# Dada Language Server Binaries

This directory contains pre-compiled binaries of the Dada Language Server for different platforms.

## Directory Structure

- `darwin-x64/`: macOS Intel binaries
- `darwin-arm64/`: macOS Apple Silicon binaries
- `linux-x64/`: Linux x64 binaries
- `win32-x64/`: Windows x64 binaries

## Adding Binaries

To add a binary for a specific platform:

1. Build the Dada Language Server for the target platform:
   ```
   cargo build --release -p dada-lsp-server
   ```

2. Copy the binary to the appropriate directory:
   - For macOS: `cp target/release/dada-lsp-server bin/darwin-x64/`
   - For Linux: `cp target/release/dada-lsp-server bin/linux-x64/`
   - For Windows: `copy target\release\dada-lsp-server.exe bin\win32-x64\`

## Automated Building

For production releases, it's recommended to set up a CI/CD pipeline to automatically build binaries for all supported platforms.

## Development

During development, if no binary is found, the extension will attempt to use `cargo run -p dada-lsp-server --` to build and run the server on-demand.
