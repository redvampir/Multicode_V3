import { ButtonWidget, PanelWidget, TextWidget } from './widgets.js';

export function widgetToCode(widget) {
  const meta = {
    id: widget.id,
    type: 'widget',
    kind: widget.kind,
    x: widget.x,
    y: widget.y,
    updated_at: new Date().toISOString(),
  };
  if (widget.kind === 'button') meta.label = widget.label;
  if (widget.kind === 'text') meta.text = widget.text;
  if (widget.kind === 'panel') { meta.w = widget.w; meta.h = widget.h; }
  const metaComment = `// @VISUAL_META ${JSON.stringify(meta)}`;
  switch (widget.kind) {
    case 'button':
      return `${metaComment}\n<button id="${widget.id}" style="position:absolute;left:${widget.x}px;top:${widget.y}px;">${widget.label}</button>`;
    case 'panel':
      return `${metaComment}\n<div id="${widget.id}" style="position:absolute;left:${widget.x}px;top:${widget.y}px;width:${widget.w}px;height:${widget.h}px;border:1px solid #333;"></div>`;
    case 'text':
      return `${metaComment}\n<span id="${widget.id}" style="position:absolute;left:${widget.x}px;top:${widget.y}px;">${widget.text}</span>`;
    default:
      return metaComment;
  }
}

export function syncWidgetsToCode(widgets, source) {
  const lines = source.split(/\r?\n/);
  widgets.forEach(w => {
    const snippetLines = widgetToCode(w).split(/\r?\n/);
    const regex = new RegExp(`@VISUAL_META\\s+\\{\\"id\\":\\"${w.id}\\"`);
    const idx = lines.findIndex(line => regex.test(line));
    if (idx !== -1) {
      lines.splice(idx, snippetLines.length, ...snippetLines);
    } else {
      if (lines.length && lines[lines.length - 1] !== '') lines.push('');
      lines.push(...snippetLines);
    }
  });
  return lines.join('\n');
}

export function widgetsFromCode(source) {
  const widgets = [];
  const regex = /@VISUAL_META\s+(\{[^}]+\})/g;
  let match;
  while ((match = regex.exec(source))) {
    try {
      const meta = JSON.parse(match[1]);
      if (meta.type !== 'widget') continue;
      let w;
      switch (meta.kind) {
        case 'button':
          w = new ButtonWidget(meta.id, meta.x, meta.y, meta.label);
          break;
        case 'panel':
          w = new PanelWidget(meta.id, meta.x, meta.y, meta.w, meta.h);
          break;
        case 'text':
          w = new TextWidget(meta.id, meta.x, meta.y, meta.text);
          break;
        default:
          continue;
      }
      widgets.push(w);
    } catch (e) {
      // ignore malformed meta
    }
  }
  return widgets;
}
