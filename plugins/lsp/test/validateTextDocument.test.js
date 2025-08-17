import { describe, it, expect, vi } from 'vitest';
const { validateTextDocument, connection } = require('../server');
const { DiagnosticSeverity } = require('vscode-languageserver/node');

describe('validateTextDocument', () => {
  it('publishes diagnostics for metadata markers', () => {
    const text = 'hello\nmeta: data\n';
    const textDocument = {
      uri: 'file:///test',
      getText: () => text,
      positionAt: (offset) => {
        const lines = text.slice(0, offset).split('\n');
        return { line: lines.length - 1, character: lines[lines.length - 1].length };
      }
    };
    const sendDiagnosticsSpy = vi.spyOn(connection, 'sendDiagnostics').mockImplementation(() => {});
    validateTextDocument(textDocument);
    expect(sendDiagnosticsSpy).toHaveBeenCalledWith({
      uri: 'file:///test',
      diagnostics: [
        {
          severity: DiagnosticSeverity.Hint,
          range: {
            start: { line: 1, character: 0 },
            end: { line: 1, character: 10 }
          },
          message: 'Metadata: data',
          source: 'metadata-lsp'
        }
      ]
    });
    sendDiagnosticsSpy.mockRestore();
  });
});
