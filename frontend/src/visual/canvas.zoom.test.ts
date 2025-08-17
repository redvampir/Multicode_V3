// @vitest-environment jsdom
import { describe, it, expect, vi } from 'vitest';
vi.mock('../editor/visual-meta.js', () => ({
  updateMetaComment: vi.fn(),
  previewDiff: vi.fn().mockResolvedValue(true),
  renameMetaId: vi.fn().mockResolvedValue(true)
}));
vi.mock('./block-editor.ts', () => ({ openBlockEditor: vi.fn() }));
import { VisualCanvas } from './canvas.js';

describe('magnifier', () => {
  it('deactivates on window blur', () => {
    const canvasEl = document.createElement('canvas');
    Object.defineProperty(canvasEl, 'clientWidth', { value: 200 });
    Object.defineProperty(canvasEl, 'clientHeight', { value: 200 });
    canvasEl.getContext = () => ({ save(){}, setTransform(){}, clearRect(){}, beginPath(){}, stroke(){}, moveTo(){}, lineTo(){}, fillRect(){}, strokeRect(){}, fillText(){}, restore(){} });
    globalThis.requestAnimationFrame = () => 0;
    const vc = new VisualCanvas(canvasEl);
    vc.magnifier.active = true;
    window.dispatchEvent(new Event('blur'));
    expect(vc.magnifier.active).toBe(false);
  });

  it('changes scale on wheel', () => {
    const canvasEl = document.createElement('canvas');
    Object.defineProperty(canvasEl, 'clientWidth', { value: 200 });
    Object.defineProperty(canvasEl, 'clientHeight', { value: 200 });
    canvasEl.getContext = () => ({ save(){}, setTransform(){}, clearRect(){}, beginPath(){}, stroke(){}, moveTo(){}, lineTo(){}, fillRect(){}, strokeRect(){}, fillText(){}, restore(){} });
    globalThis.requestAnimationFrame = () => 0;
    const vc = new VisualCanvas(canvasEl);
    vc.magnifier.active = true;
    const start = vc.magnifier.scale;
    const up = new WheelEvent('wheel', { deltaY: -100, cancelable: true });
    canvasEl.dispatchEvent(up);
    const increased = vc.magnifier.scale;
    expect(increased).toBeGreaterThan(start);
    const down = new WheelEvent('wheel', { deltaY: 100, cancelable: true });
    canvasEl.dispatchEvent(down);
    expect(vc.magnifier.scale).toBeLessThan(increased);
  });
});
