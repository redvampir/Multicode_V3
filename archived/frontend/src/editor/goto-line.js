/**
 * Prompt for a line number and move cursor to that line.
 * Highlights the line and handles invalid input.
 *
 * @param {import('@codemirror/view').EditorView} view
 */
import { t } from "../shared/i18n.ts";

export function gotoLine(view) {
  if (!view) return;
  const input = prompt(t('goto_line_prompt'));
  if (!input) return;
  const lineNumber = Number(input);
  if (!Number.isInteger(lineNumber) || lineNumber < 1 || lineNumber > view.state.doc.lines) {
    alert(t('invalid_line_number'));
    return;
  }
  const line = view.state.doc.line(lineNumber);
  view.dispatch({
    selection: { anchor: line.from, head: line.to },
    scrollIntoView: true
  });
}
