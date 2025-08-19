// @vitest-environment jsdom
import { describe, it, expect, vi } from 'vitest';

async function setup() {
  vi.resetModules();
  vi.mock('./blocks.js', () => ({
    createBlock: vi.fn((kind, id, x, y, label, color, data) => ({ kind, id, x, y, label, color, data }))
  }));
  vi.mock('./theme.ts', () => ({ getTheme: () => ({ blockKinds: {}, blockFill: '#fff', blockStroke: '#000', blockText: '#000' }) }));
  vi.mock('../editor/navigation.js', () => ({ gotoRelated: vi.fn() }));
  vi.mock('../editor/goto-line.js', () => ({ gotoLine: vi.fn() }));
  vi.mock('../../scripts/format.js', () => ({ formatCurrentFile: vi.fn() }));
  vi.mock('../editor/command-palette.ts', () => ({ openCommandPalette: vi.fn() }));
  vi.mock('./export.ts', () => ({ exportPNG: vi.fn() }));
  const mod = await import('./hotkeys.ts');
  const canvas: any = {
    blocks: [],
    blocksData: [],
    blockDataMap: new Map(),
    connections: [],
    connect: vi.fn((a, b) => canvas.connections.push([a, b])),
    selected: new Set(),
    locale: 'en',
    moveCallback: vi.fn(),
    draw: vi.fn(),
    getFreePos: vi.fn(() => ({ x: 10, y: 20 }))
  };
  mod.setCanvas(canvas);
  mod.registerHotkeys();
  return { mod, canvas };
}

describe('hotkeys block insertion', () => {
  const operatorCases = [
    { keys: ['+', 'x'], kind: 'Operator/Add' },
    { keys: ['-', 'x'], kind: 'Operator/Subtract' },
    { keys: ['+', '+'], kind: 'Op/Inc' },
    { keys: ['-', '-'], kind: 'Op/Dec' },
    { keys: ['*'], kind: 'Operator/Multiply' },
    { keys: ['/'], kind: 'Operator/Divide' },
    { keys: ['%'], kind: 'Operator/Modulo' },
    { keys: ['?', ':'], kind: 'Op/Ternary' },
    { keys: ['=', '='], kind: 'OpComparison/Equal' },
    { keys: ['!', '='], kind: 'OpComparison/NotEqual' },
    { keys: ['>', 'x'], kind: 'OpComparison/Greater' },
    { keys: ['>', '='], kind: 'OpComparison/GreaterEqual' },
    { keys: ['<', 'x'], kind: 'OpComparison/Less' },
    { keys: ['<', '='], kind: 'OpComparison/LessEqual' },
    { keys: ['&', '&'], kind: 'OpLogic/And' },
    { keys: ['|', '|'], kind: 'OpLogic/Or' },
    { keys: ['!', 'x'], kind: 'OpLogic/Not' }
  ];

  operatorCases.forEach(({ keys, kind }) => {
    it(`creates ${kind} block`, async () => {
      const { mod, canvas } = await setup();
      keys.forEach(k => document.dispatchEvent(new KeyboardEvent('keydown', { key: k })));
      expect(canvas.blocksData[0]).toMatchObject({ kind, x: 10, y: 20 });
      mod.unregisterHotkeys();
    });
  });

  const keywordCases = [
    { word: 'if', kind: 'If' },
    { word: 'switch', kind: 'Switch' },
    { word: 'for', kind: 'Loop/For' },
    { word: 'while', kind: 'Loop/While' },
    { word: 'return', kind: 'Return' },
    { word: 'var', kind: 'Variable/Get' },
    { word: 'let', kind: 'Variable/Set' }
  ];

  keywordCases.forEach(({ word, kind }) => {
    it(`creates ${kind} block when typing ${word}`, async () => {
      const { mod, canvas } = await setup();
      for (const ch of word) {
        document.dispatchEvent(new KeyboardEvent('keydown', { key: ch }));
      }
      expect(canvas.blocksData[0]).toMatchObject({ kind, x: 10, y: 20 });
      mod.unregisterHotkeys();
    });
  });

  it('auto connects log block to active output', async () => {
    const { mod, canvas } = await setup();
    const src = { id: 'src' } as any;
    canvas.blocks.push(src);
    canvas.selected = new Set([src]);
    (canvas as any).activeOutput = src;
    document.dispatchEvent(new KeyboardEvent('keydown', { key: 'l', ctrlKey: true }));
    expect(canvas.connections).toHaveLength(1);
    expect(canvas.connections[0][0]).toBe(src);
    expect(canvas.connections[0][1]).toMatchObject({ kind: 'Log' });
    mod.unregisterHotkeys();
  });

  it('undos and redoes block insertion', async () => {
    const { mod, canvas } = await setup();
    document.dispatchEvent(new KeyboardEvent('keydown', { key: 'i' }));
    document.dispatchEvent(new KeyboardEvent('keydown', { key: 'f' }));
    await Promise.resolve();
    expect(canvas.blocksData).toHaveLength(1);
    document.dispatchEvent(new KeyboardEvent('keydown', { key: 'z', ctrlKey: true }));
    await Promise.resolve();
    expect(canvas.blocksData).toHaveLength(0);
    document.dispatchEvent(new KeyboardEvent('keydown', { key: 'z', ctrlKey: true, shiftKey: true }));
    await Promise.resolve();
    expect(canvas.blocksData).toHaveLength(1);
    mod.unregisterHotkeys();
  });
});

describe('hotkey bindings', () => {
  it('allows export only with bound hotkey', async () => {
    const { mod } = await setup();
    const exp = (await import('./export.ts')).exportPNG as any;
    document.dispatchEvent(new KeyboardEvent('keydown', { key: 'E', ctrlKey: true }));
    expect(exp).not.toHaveBeenCalled();
    document.dispatchEvent(new KeyboardEvent('keydown', { key: 'E', ctrlKey: true, shiftKey: true }));
    expect(exp).toHaveBeenCalled();
    mod.unregisterHotkeys();
  });
});
