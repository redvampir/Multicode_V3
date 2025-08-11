import { readTextFile, writeTextFile } from "@tauri-apps/api/fs";
import path from "path";
import { EditorView } from "@codemirror/view";
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

  const normalizedProjectDir = path.resolve(projectDir);
  const normalizedSourcePath = path.resolve(sourcePath);

  const isInsideProject = (targetPath) => {
    const relative = path.relative(normalizedProjectDir, targetPath);
    return !relative.startsWith("..") && !path.isAbsolute(relative);
  };

  if (!isInsideProject(normalizedSourcePath)) {
    throw new Error("sourcePath is outside of projectDir");
  }

  const fileName = path.basename(normalizedSourcePath);
  const destPath = path.resolve(normalizedProjectDir, fileName);

  if (!isInsideProject(destPath)) {
    throw new Error("destPath is outside of projectDir");
  }

  let content;
  try {
    content = await readTextFile(normalizedSourcePath);
  } catch (error) {
    throw new Error(`Failed to read file at ${normalizedSourcePath}: ${error.message}`);
  }

  const meta = {
    id: crypto.randomUUID(),
    version: 1,
    x: 0,
    y: 0,
    origin: normalizedSourcePath,
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
