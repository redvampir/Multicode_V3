import { describe, it, expect } from 'vitest';
import { ButtonWidget } from './widgets.js';
import { widgetToCode, syncWidgetsToCode, widgetsFromCode } from './codegen.js';

describe('widget code generation', () => {
  it('generates code with meta comment', () => {
    const w = new ButtonWidget('b1', 10, 20, 'OK');
    const code = widgetToCode(w);
    expect(code).toContain('@VISUAL_META');
    expect(code).toContain('"type":"widget"');
  });

  it('syncs widgets into source and updates position', () => {
    const w = new ButtonWidget('b1', 10, 20, 'OK');
    let source = '';
    source = syncWidgetsToCode([w], source);
    expect(source).toMatch(/left:10px/);
    w.x = 30; w.y = 40;
    source = syncWidgetsToCode([w], source);
    expect(source).toMatch(/left:30px/);
  });

  it('parses widgets from code', () => {
    const w = new ButtonWidget('b2', 5, 5, 'Hi');
    const source = syncWidgetsToCode([w], '');
    const widgets = widgetsFromCode(source);
    expect(widgets.length).toBe(1);
    expect(widgets[0].id).toBe('b2');
    expect(widgets[0].x).toBe(5);
  });
});
