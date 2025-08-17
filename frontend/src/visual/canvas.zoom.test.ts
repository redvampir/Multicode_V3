// @vitest-environment jsdom
import { describe, it, expect, vi } from 'vitest';
vi.mock('../editor/visual-meta.js', () => ({
  updateMetaComment: vi.fn(),
  previewDiff: vi.fn().mockResolvedValue(true),
  renameMetaId: vi.fn().mockResolvedValue(true)
}));
vi.mock('./block-editor.ts', () => ({ openBlockEditor: vi.fn() }));
import { VisualCanvas } from './canvas.js';

describe('zoomToFit with micro blocks', () => {
  it('accounts for isMicro flag', () => {
    const canvasEl = document.createElement('canvas');
    Object.defineProperty(canvasEl, 'clientWidth', { value: 200 });
    Object.defineProperty(canvasEl, 'clientHeight', { value: 200 });
    canvasEl.getContext = () => ({ save(){}, setTransform(){}, clearRect(){}, beginPath(){}, stroke(){}, moveTo(){}, lineTo(){}, fillRect(){}, strokeRect(){}, fillText(){}, restore(){} });
    globalThis.requestAnimationFrame = () => 0;
    const vc = new VisualCanvas(canvasEl);
    vc.blocks = [
      { x: 0, y: 0, w: 56, h: 28, isMicro: true }
    ];
    vc.zoomToFit();
    expect(vc.scale).toBeCloseTo(1.5);
    expect(vc.offset.x).toBeCloseTo(10);
    expect(vc.offset.y).toBeCloseTo(62.5);
  });
});
