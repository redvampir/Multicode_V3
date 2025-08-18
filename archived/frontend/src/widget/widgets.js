export class Widget {
  constructor(id, x, y) {
    this.id = id;
    this.x = x;
    this.y = y;
    this.el = null;
    this.kind = 'widget';
    this.onMove = null;
  }

  createElement() {
    const el = document.createElement('div');
    el.style.position = 'absolute';
    el.style.left = this.x + 'px';
    el.style.top = this.y + 'px';
    el.dataset.widgetId = this.id;
    this.el = el;
    this.makeDraggable(el);
    return el;
  }

  makeDraggable(el) {
    let startX, startY;
    const move = e => {
      const dx = e.clientX - startX;
      const dy = e.clientY - startY;
      this.x += dx;
      this.y += dy;
      startX = e.clientX;
      startY = e.clientY;
      el.style.left = this.x + 'px';
      el.style.top = this.y + 'px';
      if (this.onMove) this.onMove(this);
    };
    el.addEventListener('mousedown', e => {
      e.preventDefault();
      startX = e.clientX;
      startY = e.clientY;
      document.addEventListener('mousemove', move);
      document.addEventListener('mouseup', () => {
        document.removeEventListener('mousemove', move);
      }, { once: true });
    });
  }

  render(parent) {
    parent.appendChild(this.createElement());
  }
}

export class ButtonWidget extends Widget {
  constructor(id, x, y, label = 'Button') {
    super(id, x, y);
    this.label = label;
    this.kind = 'button';
  }

  createElement() {
    const btn = document.createElement('button');
    btn.textContent = this.label;
    btn.style.position = 'absolute';
    btn.style.left = this.x + 'px';
    btn.style.top = this.y + 'px';
    btn.dataset.widgetId = this.id;
    this.el = btn;
    this.makeDraggable(btn);
    return btn;
  }
}

export class PanelWidget extends Widget {
  constructor(id, x, y, w = 120, h = 80) {
    super(id, x, y);
    this.w = w;
    this.h = h;
    this.kind = 'panel';
  }

  createElement() {
    const div = document.createElement('div');
    div.style.position = 'absolute';
    div.style.left = this.x + 'px';
    div.style.top = this.y + 'px';
    div.style.width = this.w + 'px';
    div.style.height = this.h + 'px';
    div.style.border = '1px solid #333';
    div.dataset.widgetId = this.id;
    this.el = div;
    this.makeDraggable(div);
    return div;
  }
}

export class TextWidget extends Widget {
  constructor(id, x, y, text = 'Text') {
    super(id, x, y);
    this.text = text;
    this.kind = 'text';
  }

  createElement() {
    const span = document.createElement('span');
    span.textContent = this.text;
    span.style.position = 'absolute';
    span.style.left = this.x + 'px';
    span.style.top = this.y + 'px';
    span.dataset.widgetId = this.id;
    this.el = span;
    this.makeDraggable(span);
    return span;
  }
}
