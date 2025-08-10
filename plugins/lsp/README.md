# Metadata LSP Prototype

This directory contains a prototype [Language Server Protocol](https://microsoft.github.io/language-server-protocol/) implementation. The server scans documents for metadata markers and exposes them to editors so they can be highlighted in other IDEs.

## Running the server

```bash
npm install
npm start
```

The server communicates over standard input/output and therefore can be embedded in any LSP‑aware editor.

## Protocol

The server uses the standard LSP messages:

* `initialize` – the client initializes the connection. The server replies with support for basic text document synchronization.
* `textDocument/didOpen` and `textDocument/didChange` – whenever a document is opened or changed the server scans its contents for `meta:` markers.
* `textDocument/publishDiagnostics` – for every occurrence of `meta:` the server publishes a diagnostic with severity `Hint`. Editors can use these diagnostics to highlight metadata ranges.

A metadata marker has the form:

```
meta: description of the data
```

The text after `meta:` is included in the diagnostic message so an IDE can render it to the user.
