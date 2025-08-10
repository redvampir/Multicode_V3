export class Block {
  constructor(x, y, w, h, label, color = '#fff') {
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

export class FunctionBlock extends Block {
  constructor(x, y) {
    super(x, y, 120, 50, 'Function', '#e0f7fa');
  }
}

export class VariableBlock extends Block {
  constructor(x, y) {
    super(x, y, 120, 50, 'Variable', '#f1f8e9');
  }
}

export class ConditionBlock extends Block {
  constructor(x, y) {
    super(x, y, 120, 50, 'Condition', '#fff9c4');
  }
}

export class LoopBlock extends Block {
  constructor(x, y) {
    super(x, y, 120, 50, 'Loop', '#fce4ec');
  }
}
