import { readTextFile, writeTextFile } from "@tauri-apps/api/fs";
import { basename, join } from "@tauri-apps/api/path";
import { insertVisualMeta } from "../editor/visual-meta.js";

/**
 * Copy an external file into the project and open it for annotation.
 * A @VISUAL_META comment with a reverse path to the original file is
 * inserted at the top of the copied file. The provided CodeMirror view
 * is focused so the user can insert additional @VISUAL_META markers.
 *
 * @param {string} sourcePath Absolute path to the external file.
 * @param {string} projectDir Project directory where the file should be copied.
 * @param {import("@codemirror/view").EditorView} view Active editor view.
 * @param {string} lang Language identifier used for visual meta templates.
 * @returns {Promise<string>} Path to the copied file inside the project.
 */
export async function annotateExternalFile(sourcePath, projectDir, view, lang) {
  const content = await readTextFile(sourcePath);
  const fileName = await basename(sourcePath);
  const destPath = await join(projectDir, fileName);

  const meta = {
    id: crypto.randomUUID(),
    x: 0,
    y: 0,
    origin: sourcePath,
  };
  const comment = `// @VISUAL_META ${JSON.stringify(meta)}\n`;
  await writeTextFile(destPath, comment + content);

  // focus the editor and allow further annotations
  view.focus();
  insertVisualMeta(view, lang);
  return destPath;
}
