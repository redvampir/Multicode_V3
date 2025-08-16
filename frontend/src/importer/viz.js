import { exists, readTextFile, writeTextFile } from "@tauri-apps/api/fs";
import path from "path";

/** Serialize a VizDocument to JSON string. */
export function serializeVizDocument(doc) {
  return JSON.stringify(doc);
}

/** Deserialize a VizDocument from JSON string. */
export function deserializeVizDocument(text) {
  return JSON.parse(text);
}

/**
 * Load VizDocument associated with source file.
 * Looks for sibling X.viz.json first, otherwise parses @viz comment from file content.
 * @param {string} sourcePath absolute path to source file
 * @returns {Promise<object|null>} parsed VizDocument or null if not found
 */
export async function loadVizDocument(sourcePath) {
  const parsed = path.parse(sourcePath);
  const vizPath = path.join(parsed.dir, `${parsed.name}.viz.json`);
  if (await exists(vizPath)) {
    const json = await readTextFile(vizPath);
    return deserializeVizDocument(json);
  }
  const content = await readTextFile(sourcePath);
  const match = content.match(/@viz\s*(\{[\s\S]*?\})/);
  if (match) {
    return deserializeVizDocument(match[1]);
  }
  return null;
}

/** Save VizDocument next to source file as X.viz.json */
export async function saveVizDocument(sourcePath, doc) {
  const parsed = path.parse(sourcePath);
  const vizPath = path.join(parsed.dir, `${parsed.name}.viz.json`);
  await writeTextFile(vizPath, serializeVizDocument(doc));
  return vizPath;
}
