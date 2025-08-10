export class Block {
  constructor(id, x, y, w, h, label, color = '#fff') {
    this.id = id;
    this.x = x;
    this.y = y;
    this.w = w;
    this.h = h;
    this.label = label;
    this.color = color;
  }

  draw(ctx) {
    ctx.fillStyle = this.color;
    ctx.strokeStyle = '#333';
    ctx.lineWidth = 2;
    ctx.fillRect(this.x, this.y, this.w, this.h);
    ctx.strokeRect(this.x, this.y, this.w, this.h);
    ctx.fillStyle = '#000';
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

export function registerBlock(kind, ctor) {
  registry[kind] = ctor;
}

export function createBlock(kind, id, x, y, label, color) {
  const Ctor = registry[kind] || Block;
  return new Ctor(id, x, y, 120, 50, label, color);
}

export async function loadBlockPlugins(urls) {
  for (const url of urls) {
    try {
      const mod = await import(/* @vite-ignore */ url);
      if (mod && typeof mod.register === 'function') {
        mod.register({ Block, registerBlock });
      }
    } catch (e) {
      console.error('Failed to load block plugin', url, e);
    }
  }
}

// ---- Built-in blocks -------------------------------------------------------

export class FunctionBlock extends Block {
  constructor(id, x, y) {
    super(id, x, y, 120, 50, 'Function', '#e0f7fa');
  }
}

export class VariableBlock extends Block {
  constructor(id, x, y) {
    super(id, x, y, 120, 50, 'Variable', '#f1f8e9');
  }
}

export class ConditionBlock extends Block {
  constructor(id, x, y) {
    super(id, x, y, 120, 50, 'Condition', '#fff9c4');
  }
}

export class LoopBlock extends Block {
  constructor(id, x, y) {
    super(id, x, y, 120, 50, 'Loop', '#fce4ec');
  }
}

registerBlock('Function', FunctionBlock);
registerBlock('Variable', VariableBlock);
registerBlock('Condition', ConditionBlock);
registerBlock('Loop', LoopBlock);
