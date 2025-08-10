import settings from '../../settings.json' assert { type: 'json' };
import { writeFile } from 'node:fs/promises';

interface PluginInfo {
  name: string;
  enabled: boolean;
}

const cfg: { plugins?: Record<string, boolean> } = settings as any;
const settingsPath = new URL('../../settings.json', import.meta.url);

function ensureSection() {
  if (!cfg.plugins) cfg.plugins = {};
}

export async function fetchPlugins(): Promise<PluginInfo[]> {
  const res = await fetch('/plugins');
  const names: string[] = await res.json();
  ensureSection();
  return names.map(name => ({ name, enabled: cfg.plugins![name] !== false }));
}

async function save() {
  await writeFile(settingsPath, JSON.stringify(settings, null, 2));
}

export async function togglePlugin(name: string, enabled: boolean): Promise<void> {
  ensureSection();
  cfg.plugins![name] = enabled;
  await save();
  await fetch('/plugins', { method: 'POST' });
}

export function isEnabled(name: string): boolean {
  ensureSection();
  return cfg.plugins![name] !== false;
}
