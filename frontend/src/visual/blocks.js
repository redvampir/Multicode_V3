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

export class NumberLiteralBlock extends Block {
  static defaultSize = { width: 120, height: 50 };
  static ports = [{ id: 'out', kind: 'data', dir: 'out' }];
  constructor(id, x, y) {
    super(
      id,
      x,
      y,
      NumberLiteralBlock.defaultSize.width,
      NumberLiteralBlock.defaultSize.height,
      'Number',
      getTheme().blockKinds.Literal
    );
    this.ports = NumberLiteralBlock.ports;
  }
}

export class StringLiteralBlock extends Block {
  static defaultSize = { width: 120, height: 50 };
  static ports = [{ id: 'out', kind: 'data', dir: 'out' }];
  constructor(id, x, y) {
    super(
      id,
      x,
      y,
      StringLiteralBlock.defaultSize.width,
      StringLiteralBlock.defaultSize.height,
      'String',
      getTheme().blockKinds.Literal
    );
    this.ports = StringLiteralBlock.ports;
  }
}

export class BooleanLiteralBlock extends Block {
  static defaultSize = { width: 120, height: 50 };
  static ports = [{ id: 'out', kind: 'data', dir: 'out' }];
  constructor(id, x, y) {
    super(
      id,
      x,
      y,
      BooleanLiteralBlock.defaultSize.width,
      BooleanLiteralBlock.defaultSize.height,
      'Boolean',
      getTheme().blockKinds.Literal
    );
    this.ports = BooleanLiteralBlock.ports;
  }
}

export class NullLiteralBlock extends Block {
  static defaultSize = { width: 120, height: 50 };
  static ports = [{ id: 'out', kind: 'data', dir: 'out' }];
  constructor(id, x, y) {
    super(
      id,
      x,
      y,
      NullLiteralBlock.defaultSize.width,
      NullLiteralBlock.defaultSize.height,
      'Null',
      getTheme().blockKinds.Literal
    );
    this.ports = NullLiteralBlock.ports;
  }
}

export class FunctionBlock extends Block {
  constructor(id, x, y) {
    super(id, x, y, 120, 50, 'Function', getTheme().blockKinds.Function);
  }
}

export class FunctionDefineBlock extends Block {
  static defaultSize = { width: 120, height: 50 };
  static ports = [
    { id: 'exec', kind: 'exec', dir: 'in' },
    { id: 'params', kind: 'data', dir: 'in' },
    { id: 'out', kind: 'exec', dir: 'out' }
  ];
  constructor(id, x, y) {
    super(
      id,
      x,
      y,
      FunctionDefineBlock.defaultSize.width,
      FunctionDefineBlock.defaultSize.height,
      'Function Define',
      getTheme().blockKinds.Function
    );
    this.ports = FunctionDefineBlock.ports;
  }
}

export class FunctionCallBlock extends Block {
  static defaultSize = { width: 120, height: 50 };
  static ports = [
    { id: 'exec', kind: 'exec', dir: 'in' },
    { id: 'params', kind: 'data', dir: 'in' },
    { id: 'out', kind: 'exec', dir: 'out' }
  ];
  constructor(id, x, y) {
    super(
      id,
      x,
      y,
      FunctionCallBlock.defaultSize.width,
      FunctionCallBlock.defaultSize.height,
      'Function Call',
      getTheme().blockKinds.Function
    );
    this.ports = FunctionCallBlock.ports;
  }
}

export class ReturnBlock extends Block {
  static defaultSize = { width: 120, height: 50 };
  static ports = [
    { id: 'exec', kind: 'exec', dir: 'in' },
    { id: 'params', kind: 'data', dir: 'in' },
    { id: 'out', kind: 'exec', dir: 'out' }
  ];
  constructor(id, x, y) {
    super(
      id,
      x,
      y,
      ReturnBlock.defaultSize.width,
      ReturnBlock.defaultSize.height,
      'Return',
      getTheme().blockKinds.Function
    );
    this.ports = ReturnBlock.ports;
  }
}

export class VariableBlock extends Block {
  constructor(id, x, y) {
    super(id, x, y, 120, 50, 'Variable', getTheme().blockKinds.Variable);
  }
}

export class VariableGetBlock extends Block {
  static defaultSize = { width: 120, height: 50 };
  static ports = [{ id: 'data', kind: 'data', dir: 'out' }];
  constructor(id, x, y) {
    super(
      id,
      x,
      y,
      VariableGetBlock.defaultSize.width,
      VariableGetBlock.defaultSize.height,
      'Variable Get',
      getTheme().blockKinds.Variable
    );
    this.ports = VariableGetBlock.ports;
  }
}

export class VariableSetBlock extends Block {
  static defaultSize = { width: 120, height: 50 };
  static ports = [
    { id: 'value', kind: 'data', dir: 'in' },
    { id: 'exec', kind: 'exec', dir: 'out' }
  ];
  constructor(id, x, y) {
    super(
      id,
      x,
      y,
      VariableSetBlock.defaultSize.width,
      VariableSetBlock.defaultSize.height,
      'Variable Set',
      getTheme().blockKinds.Variable
    );
    this.ports = VariableSetBlock.ports;
  }
}

export class ConditionBlock extends Block {
  constructor(id, x, y) {
    super(id, x, y, 120, 50, 'Condition', getTheme().blockKinds.Condition);
  }
}

export class IfBlock extends Block {
  static defaultSize = { width: 120, height: 50 };
  static ports = [
    { id: 'cond', kind: 'data', dir: 'in' },
    { id: 'exec', kind: 'exec', dir: 'in' },
    { id: 'then', kind: 'exec', dir: 'out' },
    { id: 'else', kind: 'exec', dir: 'out' }
  ];
  constructor(id, x, y) {
    super(
      id,
      x,
      y,
      IfBlock.defaultSize.width,
      IfBlock.defaultSize.height,
      'If',
      getTheme().blockKinds.If
    );
    this.ports = IfBlock.ports;
  }
}

export class LoopBlock extends Block {
  constructor(id, x, y) {
    super(id, x, y, 120, 50, 'Loop', getTheme().blockKinds.Loop);
  }
}

export class ArrayNewBlock extends Block {
  static defaultSize = { width: 120, height: 50 };
  static ports = [
    { id: 'execIn', kind: 'exec', dir: 'in' },
    { id: 'execOut', kind: 'exec', dir: 'out' },
    { id: 'out', kind: 'data', dir: 'out' }
  ];
  constructor(id, x, y) {
    super(
      id,
      x,
      y,
      ArrayNewBlock.defaultSize.width,
      ArrayNewBlock.defaultSize.height,
      'Array New',
      getTheme().blockKinds.Array
    );
    this.ports = ArrayNewBlock.ports;
  }
}

export class ArrayGetBlock extends Block {
  static defaultSize = { width: 120, height: 50 };
  static ports = [
    { id: 'execIn', kind: 'exec', dir: 'in' },
    { id: 'execOut', kind: 'exec', dir: 'out' },
    { id: 'array', kind: 'data', dir: 'in' },
    { id: 'index', kind: 'data', dir: 'in' },
    { id: 'value', kind: 'data', dir: 'out' }
  ];
  constructor(id, x, y) {
    super(
      id,
      x,
      y,
      ArrayGetBlock.defaultSize.width,
      ArrayGetBlock.defaultSize.height,
      'Array Get',
      getTheme().blockKinds.Array
    );
    this.ports = ArrayGetBlock.ports;
  }
}

export class ArraySetBlock extends Block {
  static defaultSize = { width: 120, height: 50 };
  static ports = [
    { id: 'execIn', kind: 'exec', dir: 'in' },
    { id: 'execOut', kind: 'exec', dir: 'out' },
    { id: 'array', kind: 'data', dir: 'in' },
    { id: 'index', kind: 'data', dir: 'in' },
    { id: 'value', kind: 'data', dir: 'in' },
    { id: 'result', kind: 'data', dir: 'out' }
  ];
  constructor(id, x, y) {
    super(
      id,
      x,
      y,
      ArraySetBlock.defaultSize.width,
      ArraySetBlock.defaultSize.height,
      'Array Set',
      getTheme().blockKinds.Array
    );
    this.ports = ArraySetBlock.ports;
  }
}

export class MapNewBlock extends Block {
  static defaultSize = { width: 120, height: 50 };
  static ports = [
    { id: 'execIn', kind: 'exec', dir: 'in' },
    { id: 'execOut', kind: 'exec', dir: 'out' },
    { id: 'out', kind: 'data', dir: 'out' }
  ];
  constructor(id, x, y) {
    super(
      id,
      x,
      y,
      MapNewBlock.defaultSize.width,
      MapNewBlock.defaultSize.height,
      'Map New',
      getTheme().blockKinds.Map
    );
    this.ports = MapNewBlock.ports;
  }
}

export class MapGetBlock extends Block {
  static defaultSize = { width: 120, height: 50 };
  static ports = [
    { id: 'execIn', kind: 'exec', dir: 'in' },
    { id: 'execOut', kind: 'exec', dir: 'out' },
    { id: 'map', kind: 'data', dir: 'in' },
    { id: 'key', kind: 'data', dir: 'in' },
    { id: 'value', kind: 'data', dir: 'out' }
  ];
  constructor(id, x, y) {
    super(
      id,
      x,
      y,
      MapGetBlock.defaultSize.width,
      MapGetBlock.defaultSize.height,
      'Map Get',
      getTheme().blockKinds.Map
    );
    this.ports = MapGetBlock.ports;
  }
}

export class MapSetBlock extends Block {
  static defaultSize = { width: 120, height: 50 };
  static ports = [
    { id: 'execIn', kind: 'exec', dir: 'in' },
    { id: 'execOut', kind: 'exec', dir: 'out' },
    { id: 'map', kind: 'data', dir: 'in' },
    { id: 'key', kind: 'data', dir: 'in' },
    { id: 'value', kind: 'data', dir: 'in' },
    { id: 'result', kind: 'data', dir: 'out' }
  ];
  constructor(id, x, y) {
    super(
      id,
      x,
      y,
      MapSetBlock.defaultSize.width,
      MapSetBlock.defaultSize.height,
      'Map Set',
      getTheme().blockKinds.Map
    );
    this.ports = MapSetBlock.ports;
  }
}

export class StructBlock extends Block {
  static defaultSize = { width: 120, height: 50 };
  static ports = [
    { id: 'execIn', kind: 'exec', dir: 'in' },
    { id: 'execOut', kind: 'exec', dir: 'out' },
    { id: 'out', kind: 'data', dir: 'out' }
  ];
  constructor(id, x, y) {
    super(
      id,
      x,
      y,
      StructBlock.defaultSize.width,
      StructBlock.defaultSize.height,
      'Struct',
      getTheme().blockKinds.Struct
    );
    this.ports = StructBlock.ports;
  }
}

registerBlock('Literal/Number', NumberLiteralBlock);
registerBlock('Literal/String', StringLiteralBlock);
registerBlock('Literal/Boolean', BooleanLiteralBlock);
registerBlock('Literal/Null', NullLiteralBlock);
registerBlock('Function', FunctionBlock);
registerBlock('Function/Define', FunctionDefineBlock);
registerBlock('Function/Call', FunctionCallBlock);
registerBlock('Return', ReturnBlock);
registerBlock('Variable', VariableBlock);
registerBlock('Variable/Get', VariableGetBlock);
registerBlock('Variable/Set', VariableSetBlock);
registerBlock('Condition', ConditionBlock);
registerBlock('If', IfBlock);
registerBlock('Loop', LoopBlock);
registerBlock('Array/New', ArrayNewBlock);
registerBlock('Array/Get', ArrayGetBlock);
registerBlock('Array/Set', ArraySetBlock);
registerBlock('Map/New', MapNewBlock);
registerBlock('Map/Get', MapGetBlock);
registerBlock('Map/Set', MapSetBlock);
registerBlock('Struct', StructBlock);
