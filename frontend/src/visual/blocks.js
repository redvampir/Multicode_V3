import { getTheme } from './theme.ts';

export class Block {
  constructor(id, x, y, w, h, label, color = getTheme().blockFill) {
    this.id = id;
    this.x = x;
    this.y = y;
    this.w = w;
    this.h = h;
    this.label = label;
    this.color = color;
  }

  draw(ctx) {
    const theme = getTheme();
    ctx.fillStyle = this.color;
    ctx.strokeStyle = theme.blockStroke;
    ctx.lineWidth = 2;
    ctx.fillRect(this.x, this.y, this.w, this.h);
    ctx.strokeRect(this.x, this.y, this.w, this.h);
    ctx.fillStyle = theme.blockText;
    ctx.font = '16px sans-serif';
    ctx.textAlign = 'center';
    ctx.textBaseline = 'middle';
    ctx.fillText(this.label, this.x + this.w / 2, this.y + this.h / 2);
  }

  contains(px, py) {
    return px >= this.x && px <= this.x + this.w &&
           py >= this.y && py <= this.y + this.h;
  }

  center() {
    return { x: this.x + this.w / 2, y: this.y + this.h / 2 };
  }
}

// ---- Plugin infrastructure -------------------------------------------------

const registry = {};
const pluginKinds = new Map(); // url -> Set of kinds
let currentPluginUrl = null;

export function registerBlock(kind, ctor) {
  registry[kind] = ctor;
  if (currentPluginUrl) {
    let kinds = pluginKinds.get(currentPluginUrl);
    if (!kinds) {
      kinds = new Set();
      pluginKinds.set(currentPluginUrl, kinds);
    }
    kinds.add(kind);
  }
}

export function unregisterBlock(kind) {
  delete registry[kind];
}

export function createBlock(kind, id, x, y, label, color) {
  const Ctor = registry[kind] || Block;
  return new Ctor(id, x, y, 120, 50, label, color);
}

async function importPlugin(url, forceReload = false) {
  if (forceReload && pluginKinds.has(url)) {
    for (const kind of pluginKinds.get(url)) {
      unregisterBlock(kind);
    }
    pluginKinds.delete(url);
  }
  const importUrl = forceReload ? `${url}?t=${Date.now()}` : url;
  try {
    currentPluginUrl = url;
    const mod = await import(/* @vite-ignore */ importUrl);
    if (mod && typeof mod.register === 'function') {
      mod.register({ Block, registerBlock });
    }
  } catch (e) {
    console.error('Failed to load block plugin', url, e);
  } finally {
    currentPluginUrl = null;
  }
}

export async function loadBlockPlugins(urls) {
  for (const url of urls) {
    const reload = pluginKinds.has(url);
    await importPlugin(url, reload);
  }
}

export async function reloadPlugins(urls) {
  for (const url of urls) {
    await importPlugin(url, true);
  }
}

// ---- Built-in blocks -------------------------------------------------------

export class FunctionBlock extends Block {
  constructor(id, x, y) {
    super(id, x, y, 120, 50, 'Function', getTheme().blockKinds.Function);
  }
}

export class VariableBlock extends Block {
  constructor(id, x, y) {
    super(id, x, y, 120, 50, 'Variable', getTheme().blockKinds.Variable);
  }
}

export class ConditionBlock extends Block {
  constructor(id, x, y) {
    super(id, x, y, 120, 50, 'Condition', getTheme().blockKinds.Condition);
  }
}

export class LoopBlock extends Block {
  constructor(id, x, y) {
    super(id, x, y, 120, 50, 'Loop', getTheme().blockKinds.Loop);
  }
}

registerBlock('Function', FunctionBlock);
registerBlock('Variable', VariableBlock);
registerBlock('Condition', ConditionBlock);
registerBlock('Loop', LoopBlock);
