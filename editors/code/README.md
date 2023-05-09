# Dada Language Server

This vscode extension provides language support for the [Dada programming language](https://dada-lang.org/).

The extension is not yet published, as changes to the language and tooling may happen frequently.

## Prerequisites
* [Node](https://docs.npmjs.com/downloading-and-installing-node-js-and-npm)

## Installation

1. Add Dada to your PATH. E.g.:
    ```bash
    cargo install --path .
    ```
    or<br></br>
    set the Dada executable path in vscode settings. E.g.:

    ```json
    "dadaLanguageServer.compiler.executablePath": "/path/to/dada"
    ```
2. Install deps in the editors/code directory: `npm install`
3. Build and package the extension: `npm run package`
4. Install the extension: `code --install-extension dada.vsix`
5. Restart vscode or reload from the command palette: (cmd/ctrl + shift + p) Developer: Reload Window
6. Open a `.dada` file.

## License

Licensed under either of [Apache License, Version 2.0][apache] or [MIT license][mit] at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this repository by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

[apache]: LICENSE
[mit]: LICENSE
