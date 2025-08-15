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

export function createBlock(kind, id, x, y, label, color, data) {
  const Ctor = registry[kind] || Block;
  return new Ctor(id, x, y, 120, 50, label, color, data);
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

export class GroupBlock extends Block {
  constructor(id, x, y, w = 200, h = 150, label = '', color = getTheme().blockStroke) {
    super(id, x, y, w, h, label, color);
  }

  draw(ctx) {
    ctx.strokeStyle = this.color;
    ctx.lineWidth = 2;
    ctx.strokeRect(this.x, this.y, this.w, this.h);
    if (this.label) {
      ctx.fillStyle = this.color;
      ctx.font = '16px sans-serif';
      ctx.textAlign = 'left';
      ctx.textBaseline = 'top';
      ctx.fillText(this.label, this.x + 4, this.y + 4);
    }
  }
}

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

export class LogBlock extends Block {
  static defaultSize = { width: 120, height: 50 };
  constructor(id, x, y, _w, _h, label, color, data) {
    super(
      id,
      x,
      y,
      LogBlock.defaultSize.width,
      LogBlock.defaultSize.height,
      label || 'Log',
      color ?? getTheme().blockKinds.Log
    );
    this.exec = !!data?.exec;
    this.updatePorts();
  }

  updatePorts() {
    this.ports = [
      { id: 'data', kind: 'data', dir: 'in' },
      ...(this.exec
        ? [
            { id: 'exec', kind: 'exec', dir: 'in' },
            { id: 'out', kind: 'exec', dir: 'out' }
          ]
        : [])
    ];
  }
}

class OperatorBlockBase extends Block {
  static defaultSize = { width: 120, height: 50 };
  static ports = [
    { id: 'lhs', kind: 'data', dir: 'in' },
    { id: 'rhs', kind: 'data', dir: 'in' },
    { id: 'out', kind: 'data', dir: 'out' }
  ];
  constructor(id, x, y, label) {
    super(
      id,
      x,
      y,
      OperatorBlockBase.defaultSize.width,
      OperatorBlockBase.defaultSize.height,
      label,
      getTheme().blockKinds.Operator || getTheme().blockFill
    );
    this.ports = OperatorBlockBase.ports;
  }
}

export class AddBlock extends OperatorBlockBase {
  constructor(id, x, y) {
    super(id, x, y, '+');
  }
}

export class SubtractBlock extends OperatorBlockBase {
  constructor(id, x, y) {
    super(id, x, y, '-');
  }
}

export class MultiplyBlock extends OperatorBlockBase {
  constructor(id, x, y) {
    super(id, x, y, '*');
  }
}

export class DivideBlock extends OperatorBlockBase {
  constructor(id, x, y) {
    super(id, x, y, '/');
  }
}

export class ModuloBlock extends OperatorBlockBase {
  constructor(id, x, y) {
    super(id, x, y, '%');
  }
}

export class OpConcatBlock extends OperatorBlockBase {
  constructor(id, x, y) {
    super(id, x, y, '++');
  }
}

class LogicOperatorBlockBase extends Block {
  static defaultSize = { width: 120, height: 50 };
  static ports = [
    { id: 'lhs', kind: 'data', dir: 'in' },
    { id: 'rhs', kind: 'data', dir: 'in' },
    { id: 'out', kind: 'data', dir: 'out' }
  ];
  constructor(id, x, y, label) {
    super(
      id,
      x,
      y,
      LogicOperatorBlockBase.defaultSize.width,
      LogicOperatorBlockBase.defaultSize.height,
      label,
      getTheme().blockKinds.OpLogic || getTheme().blockFill
    );
    this.ports = LogicOperatorBlockBase.ports;
  }
}

export class OpAndBlock extends LogicOperatorBlockBase {
  constructor(id, x, y) {
    super(id, x, y, '&&');
  }
}

export class OpOrBlock extends LogicOperatorBlockBase {
  constructor(id, x, y) {
    super(id, x, y, '||');
  }
}

export class OpNotBlock extends Block {
  static defaultSize = { width: 120, height: 50 };
  static ports = [
    { id: 'value', kind: 'data', dir: 'in' },
    { id: 'out', kind: 'data', dir: 'out' }
  ];
  constructor(id, x, y) {
    super(
      id,
      x,
      y,
      OpNotBlock.defaultSize.width,
      OpNotBlock.defaultSize.height,
      '!',
      getTheme().blockKinds.OpLogic || getTheme().blockFill
    );
    this.ports = OpNotBlock.ports;
  }
}

class ComparisonOperatorBlockBase extends Block {
  static defaultSize = { width: 120, height: 50 };
  static ports = [
    { id: 'lhs', kind: 'data', dir: 'in' },
    { id: 'rhs', kind: 'data', dir: 'in' },
    { id: 'out', kind: 'data', dir: 'out' }
  ];
  constructor(id, x, y, label) {
    super(
      id,
      x,
      y,
      ComparisonOperatorBlockBase.defaultSize.width,
      ComparisonOperatorBlockBase.defaultSize.height,
      label,
      getTheme().blockKinds.OpComparison || getTheme().blockFill
    );
    this.ports = ComparisonOperatorBlockBase.ports;
  }
}

export class OpEqualBlock extends ComparisonOperatorBlockBase {
  constructor(id, x, y) {
    super(id, x, y, '==');
  }
}

export class OpNotEqualBlock extends ComparisonOperatorBlockBase {
  constructor(id, x, y) {
    super(id, x, y, '!=');
  }
}

export class OpGreaterBlock extends ComparisonOperatorBlockBase {
  constructor(id, x, y) {
    super(id, x, y, '>');
  }
}

export class OpGreaterEqualBlock extends ComparisonOperatorBlockBase {
  constructor(id, x, y) {
    super(id, x, y, '>=');
  }
}

export class OpLessBlock extends ComparisonOperatorBlockBase {
  constructor(id, x, y) {
    super(id, x, y, '<');
  }
}

export class OpLessEqualBlock extends ComparisonOperatorBlockBase {
  constructor(id, x, y) {
    super(id, x, y, '<=');
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

export class SequenceBlock extends Block {
  static defaultSize = { width: 120, height: 50 };
  constructor(id, x, y, _w, _h, label, color, data) {
    super(
      id,
      x,
      y,
      SequenceBlock.defaultSize.width,
      SequenceBlock.defaultSize.height,
      label || 'Sequence',
      color ?? getTheme().blockKinds.Sequence
    );
    this.steps = Array.isArray(data?.steps) ? data.steps : [];
    this.updatePorts();
  }

  updatePorts() {
    this.ports = [
      ...this.steps.map(s => ({ id: `exec[${s}]`, kind: 'exec', dir: 'in' })),
      { id: 'out', kind: 'exec', dir: 'out' }
    ];
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

export class SwitchBlock extends Block {
  static defaultSize = { width: 120, height: 50 };
  constructor(id, x, y, _w, _h, label, color, data) {
    super(
      id,
      x,
      y,
      SwitchBlock.defaultSize.width,
      SwitchBlock.defaultSize.height,
      label || 'Switch',
      color ?? getTheme().blockKinds.Switch
    );
    this.cases = Array.isArray(data?.cases) ? data.cases : [];
    this.updatePorts();
  }

  updatePorts() {
    this.ports = [
      { id: 'value', kind: 'data', dir: 'in' },
      { id: 'exec', kind: 'exec', dir: 'in' },
      ...this.cases.map(c => ({ id: `case[${c}]`, kind: 'exec', dir: 'out' })),
      { id: 'default', kind: 'exec', dir: 'out' }
    ];
  }
}

export class TryBlock extends Block {
  static defaultSize = { width: 120, height: 50 };
  static ports = [
    { id: 'exec', kind: 'exec', dir: 'in' },
    { id: 'try', kind: 'exec', dir: 'out' },
    { id: 'catch', kind: 'exec', dir: 'out' },
    { id: 'finally', kind: 'exec', dir: 'out' }
  ];
  constructor(id, x, y) {
    super(
      id,
      x,
      y,
      TryBlock.defaultSize.width,
      TryBlock.defaultSize.height,
      'Try',
      getTheme().blockKinds.Try
    );
    this.ports = TryBlock.ports;
    this.exceptions = [];
  }
}

export class AwaitBlock extends Block {
  static defaultSize = { width: 120, height: 50 };
  static ports = [
    { id: 'execIn', kind: 'exec', dir: 'in' },
    { id: 'value', kind: 'data', dir: 'in' },
    { id: 'execOut', kind: 'exec', dir: 'out' },
    { id: 'result', kind: 'data', dir: 'out' }
  ];
  constructor(id, x, y) {
    super(
      id,
      x,
      y,
      AwaitBlock.defaultSize.width,
      AwaitBlock.defaultSize.height,
      'Await',
      getTheme().blockKinds.Async
    );
    this.ports = AwaitBlock.ports;
  }
}

export class DelayBlock extends Block {
  static defaultSize = { width: 120, height: 50 };
  static ports = [
    { id: 'execIn', kind: 'exec', dir: 'in' },
    { id: 'time', kind: 'data', dir: 'in' },
    { id: 'execOut', kind: 'exec', dir: 'out' }
  ];
  constructor(id, x, y) {
    super(
      id,
      x,
      y,
      DelayBlock.defaultSize.width,
      DelayBlock.defaultSize.height,
      'Delay',
      getTheme().blockKinds.Async
    );
    this.ports = DelayBlock.ports;
  }
}

export class EventOnBlock extends Block {
  static defaultSize = { width: 120, height: 50 };
  static ports = [
    { id: 'target', kind: 'data', dir: 'in' },
    { id: 'event', kind: 'data', dir: 'in' },
    { id: 'exec', kind: 'exec', dir: 'out' },
    { id: 'data', kind: 'data', dir: 'out' }
  ];
  constructor(id, x, y) {
    super(
      id,
      x,
      y,
      EventOnBlock.defaultSize.width,
      EventOnBlock.defaultSize.height,
      'Event On',
      getTheme().blockKinds.Async
    );
    this.ports = EventOnBlock.ports;
  }
}

export class LoopBlock extends Block {
  constructor(id, x, y) {
    super(id, x, y, 120, 50, 'Loop', getTheme().blockKinds.Loop);
  }
}

export class ForLoopBlock extends Block {
  static defaultSize = { width: 120, height: 50 };
  static ports = [
    { id: 'execIn', kind: 'exec', dir: 'in' },
    { id: 'execOut', kind: 'exec', dir: 'out' },
    { id: 'init', kind: 'data', dir: 'in' },
    { id: 'cond', kind: 'data', dir: 'in' },
    { id: 'iter', kind: 'data', dir: 'in' }
  ];
  constructor(id, x, y) {
    super(
      id,
      x,
      y,
      ForLoopBlock.defaultSize.width,
      ForLoopBlock.defaultSize.height,
      'For',
      getTheme().blockKinds.Loop
    );
    this.ports = ForLoopBlock.ports;
  }
}

export class WhileLoopBlock extends Block {
  static defaultSize = { width: 120, height: 50 };
  static ports = [
    { id: 'execIn', kind: 'exec', dir: 'in' },
    { id: 'execOut', kind: 'exec', dir: 'out' },
    { id: 'init', kind: 'data', dir: 'in' },
    { id: 'cond', kind: 'data', dir: 'in' },
    { id: 'iter', kind: 'data', dir: 'in' }
  ];
  constructor(id, x, y) {
    super(
      id,
      x,
      y,
      WhileLoopBlock.defaultSize.width,
      WhileLoopBlock.defaultSize.height,
      'While',
      getTheme().blockKinds.Loop
    );
    this.ports = WhileLoopBlock.ports;
  }
}

export class ForEachLoopBlock extends Block {
  static defaultSize = { width: 120, height: 50 };
  static ports = [
    { id: 'execIn', kind: 'exec', dir: 'in' },
    { id: 'execOut', kind: 'exec', dir: 'out' },
    { id: 'init', kind: 'data', dir: 'in' },
    { id: 'cond', kind: 'data', dir: 'in' },
    { id: 'iter', kind: 'data', dir: 'in' }
  ];
  constructor(id, x, y) {
    super(
      id,
      x,
      y,
      ForEachLoopBlock.defaultSize.width,
      ForEachLoopBlock.defaultSize.height,
      'For Each',
      getTheme().blockKinds.Loop
    );
    this.ports = ForEachLoopBlock.ports;
  }
}

export class BreakBlock extends Block {
  static defaultSize = { width: 120, height: 50 };
  static ports = [
    { id: 'execIn', kind: 'exec', dir: 'in' },
    { id: 'execOut', kind: 'exec', dir: 'out' }
  ];
  constructor(id, x, y) {
    super(id, x, y, BreakBlock.defaultSize.width, BreakBlock.defaultSize.height, 'Break', getTheme().blockKinds.Loop);
    this.ports = BreakBlock.ports;
  }
}

export class ContinueBlock extends Block {
  static defaultSize = { width: 120, height: 50 };
  static ports = [
    { id: 'execIn', kind: 'exec', dir: 'in' },
    { id: 'execOut', kind: 'exec', dir: 'out' }
  ];
  constructor(id, x, y) {
    super(id, x, y, ContinueBlock.defaultSize.width, ContinueBlock.defaultSize.height, 'Continue', getTheme().blockKinds.Loop);
    this.ports = ContinueBlock.ports;
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
registerBlock('Operator/Add', AddBlock);
registerBlock('Operator/Subtract', SubtractBlock);
registerBlock('Operator/Multiply', MultiplyBlock);
registerBlock('Operator/Divide', DivideBlock);
registerBlock('Operator/Modulo', ModuloBlock);
registerBlock('Operator/Concat', OpConcatBlock);
registerBlock('OpComparison/Equal', OpEqualBlock);
registerBlock('OpComparison/NotEqual', OpNotEqualBlock);
registerBlock('OpComparison/Greater', OpGreaterBlock);
registerBlock('OpComparison/GreaterEqual', OpGreaterEqualBlock);
registerBlock('OpComparison/Less', OpLessBlock);
registerBlock('OpComparison/LessEqual', OpLessEqualBlock);
registerBlock('OpLogic/And', OpAndBlock);
registerBlock('OpLogic/Or', OpOrBlock);
registerBlock('OpLogic/Not', OpNotBlock);
registerBlock('Function', FunctionBlock);
registerBlock('Function/Define', FunctionDefineBlock);
registerBlock('Function/Call', FunctionCallBlock);
registerBlock('Return', ReturnBlock);
registerBlock('Log', LogBlock);
registerBlock('Variable', VariableBlock);
registerBlock('Variable/Get', VariableGetBlock);
registerBlock('Variable/Set', VariableSetBlock);
registerBlock('Condition', ConditionBlock);
registerBlock('Sequence', SequenceBlock);
registerBlock('If', IfBlock);
registerBlock('Switch', SwitchBlock);
registerBlock('Try', TryBlock);
registerBlock('Async/Await', AwaitBlock);
registerBlock('Async/Delay', DelayBlock);
registerBlock('Async/EventOn', EventOnBlock);
registerBlock('Loop', LoopBlock);
registerBlock('Loop/For', ForLoopBlock);
registerBlock('Loop/While', WhileLoopBlock);
registerBlock('Loop/ForEach', ForEachLoopBlock);
registerBlock('Loop/Break', BreakBlock);
registerBlock('Loop/Continue', ContinueBlock);
registerBlock('Array/New', ArrayNewBlock);
registerBlock('Array/Get', ArrayGetBlock);
registerBlock('Array/Set', ArraySetBlock);
registerBlock('Map/New', MapNewBlock);
registerBlock('Map/Get', MapGetBlock);
registerBlock('Map/Set', MapSetBlock);
registerBlock('Struct', StructBlock);
registerBlock('Group', GroupBlock);
