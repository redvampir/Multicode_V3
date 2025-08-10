const {
  createConnection,
  TextDocuments,
  ProposedFeatures,
  DiagnosticSeverity
} = require('vscode-languageserver/node');

// Create a connection for the server using Node's IPC as a transport.
const connection = createConnection(ProposedFeatures.all);
const documents = new TextDocuments();

// The server's capabilities on initialization.
connection.onInitialize(() => ({
  capabilities: {
    textDocumentSync: documents.syncKind,
  }
}));

// Scan the document for `meta:` markers and publish them as diagnostics.
function validateTextDocument(textDocument) {
  const text = textDocument.getText();
  const pattern = /meta:\s*(.*)/g;
  let diagnostics = [];
  let match;
  while ((match = pattern.exec(text))) {
    const start = textDocument.positionAt(match.index);
    const end = textDocument.positionAt(match.index + match[0].length);
    diagnostics.push({
      severity: DiagnosticSeverity.Hint,
      range: { start, end },
      message: `Metadata: ${match[1].trim()}`,
      source: 'metadata-lsp'
    });
  }
  connection.sendDiagnostics({ uri: textDocument.uri, diagnostics });
}

documents.onDidOpen(event => validateTextDocument(event.document));
documents.onDidChangeContent(change => validateTextDocument(change.document));

// Make the text document manager listen on the connection
// for open, change and close text document events
// and the connection listen on the input for 
// `initialize` requests.
documents.listen(connection);
connection.listen();
