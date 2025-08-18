import { ButtonWidget, PanelWidget, TextWidget, syncWidgetsToCode } from '../../frontend/src/widget/index.js';

const canvas = document.getElementById('canvas');

const btn = new ButtonWidget('btn1', 20, 20, 'Нажать');
const panel = new PanelWidget('panel1', 100, 80, 120, 80);
const text = new TextWidget('text1', 40, 180, 'Привет');

const widgets = [btn, panel, text];
widgets.forEach(w => {
  w.onMove = () => {
    const code = syncWidgetsToCode(widgets, '');
    console.log(code);
  };
  w.render(canvas);
});
