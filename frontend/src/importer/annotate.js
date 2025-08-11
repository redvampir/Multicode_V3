import { readTextFile, writeTextFile } from "@tauri-apps/api/fs";
import { basename, join } from "@tauri-apps/api/path";
import { EditorView } from "https://cdn.jsdelivr.net/npm/@codemirror/view@6.21.3/dist/index.js";
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
  if (typeof sourcePath !== "string") {
    throw new TypeError("sourcePath must be a string");
  }
  if (typeof projectDir !== "string") {
    throw new TypeError("projectDir must be a string");
  }
  if (!(view instanceof EditorView)) {
    throw new TypeError("view must be an instance of EditorView");
  }
  if (typeof lang !== "string") {
    throw new TypeError("lang must be a string");
  }

  let content;
  try {
    content = await readTextFile(sourcePath);
  } catch (error) {
    throw new Error(`Failed to read file at ${sourcePath}: ${error.message}`);
  }

  let fileName;
  try {
    fileName = await basename(sourcePath);
  } catch (error) {
    throw new Error(`Failed to get basename for ${sourcePath}: ${error.message}`);
  }

  let destPath;
  try {
    destPath = await join(projectDir, fileName);
  } catch (error) {
    throw new Error(`Failed to join path for ${projectDir}: ${error.message}`);
  }

  const meta = {
    id: crypto.randomUUID(),
    version: 1,
    x: 0,
    y: 0,
    origin: sourcePath,
  };
  const comment = `// @VISUAL_META ${JSON.stringify(meta)}\n`;
  try {
    await writeTextFile(destPath, comment + content);
  } catch (error) {
    throw new Error(`Failed to write file at ${destPath}: ${error.message}`);
  }

  // focus the editor and allow further annotations
  view.focus();
  insertVisualMeta(view, lang);
  return destPath;
}
