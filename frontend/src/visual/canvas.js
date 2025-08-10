import { FunctionBlock, VariableBlock, ConditionBlock, LoopBlock } from './blocks.js';

export class VisualCanvas {
  constructor(canvas) {
    this.canvas = canvas;
    this.ctx = canvas.getContext('2d');
    this.scale = 1;
    this.offset = { x: 0, y: 0 };
    this.blocks = [];
    this.connections = [];
    this.dragged = null;
    this.dragOffset = { x: 0, y: 0 };
    this.panning = false;
    this.panStart = { x: 0, y: 0 };

    this.resize();
    window.addEventListener('resize', () => this.resize());
    this.registerEvents();
    requestAnimationFrame(() => this.draw());
  }

  addBlock(block) {
    this.blocks.push(block);
  }

  connect(a, b) {
    this.connections.push([a, b]);
  }

  registerEvents() {
    this.canvas.addEventListener('mousedown', e => {
      const pos = this.toWorld(e.offsetX, e.offsetY);
      this.dragged = this.blocks.find(b => b.contains(pos.x, pos.y));
      if (this.dragged) {
        this.dragOffset.x = pos.x - this.dragged.x;
        this.dragOffset.y = pos.y - this.dragged.y;
      } else {
        this.panning = true;
        this.panStart.x = e.offsetX - this.offset.x;
        this.panStart.y = e.offsetY - this.offset.y;
      }
    });

    this.canvas.addEventListener('mousemove', e => {
      const pos = this.toWorld(e.offsetX, e.offsetY);
      if (this.dragged) {
        this.dragged.x = pos.x - this.dragOffset.x;
        this.dragged.y = pos.y - this.dragOffset.y;
      } else if (this.panning) {
        this.offset.x = e.offsetX - this.panStart.x;
        this.offset.y = e.offsetY - this.panStart.y;
      }
    });

    window.addEventListener('mouseup', () => {
      this.dragged = null;
      this.panning = false;
    });

    this.canvas.addEventListener('wheel', e => {
      e.preventDefault();
      const mouseX = e.offsetX;
      const mouseY = e.offsetY;
      const worldPos = this.toWorld(mouseX, mouseY);
      const scaleFactor = e.deltaY < 0 ? 1.1 : 0.9;
      this.scale *= scaleFactor;
      const newScreenX = worldPos.x * this.scale + this.offset.x;
      const newScreenY = worldPos.y * this.scale + this.offset.y;
      this.offset.x += mouseX - newScreenX;
      this.offset.y += mouseY - newScreenY;
    });
  }

  resize() {
    this.canvas.width = this.canvas.clientWidth;
    this.canvas.height = this.canvas.clientHeight;
  }

  toWorld(x, y) {
    return {
      x: (x - this.offset.x) / this.scale,
      y: (y - this.offset.y) / this.scale
    };
  }

  draw() {
    this.ctx.save();
    this.ctx.setTransform(this.scale, 0, 0, this.scale, this.offset.x, this.offset.y);
    this.ctx.clearRect(-this.offset.x / this.scale, -this.offset.y / this.scale,
      this.canvas.width / this.scale, this.canvas.height / this.scale);

    // Draw connections
    this.ctx.strokeStyle = '#000';
    this.connections.forEach(([a, b]) => {
      const ac = a.center();
      const bc = b.center();
      this.ctx.beginPath();
      this.ctx.moveTo(ac.x, ac.y);
      this.ctx.lineTo(bc.x, bc.y);
      this.ctx.stroke();
    });

    // Draw blocks
    this.blocks.forEach(b => b.draw(this.ctx));

    this.ctx.restore();
    requestAnimationFrame(() => this.draw());
  }
}
