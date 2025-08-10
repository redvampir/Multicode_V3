import { VisualCanvas } from './canvas.js';
import { FunctionBlock, VariableBlock, ConditionBlock, LoopBlock } from './blocks.js';

document.addEventListener('DOMContentLoaded', () => {
  const canvas = document.getElementById('visual-canvas');
  if (!canvas) return;
  const vc = new VisualCanvas(canvas);

  const func = new FunctionBlock(50, 50);
  const variable = new VariableBlock(250, 50);
  const cond = new ConditionBlock(50, 200);
  const loop = new LoopBlock(250, 200);

  vc.addBlock(func);
  vc.addBlock(variable);
  vc.addBlock(cond);
  vc.addBlock(loop);

  vc.connect(func, variable);
  vc.connect(cond, loop);

  // expose for debugging
  window.visualCanvas = vc;
});
