import { t } from "../shared/i18n.ts";

interface Step {
  element: string;
  i18nKey: string;
}

const steps: Step[] = [
  { element: '#visual-canvas', i18nKey: 'tutorial_step_canvas' },
  { element: '#controls', i18nKey: 'tutorial_step_controls' },
  { element: '#search-panel', i18nKey: 'tutorial_step_search' },
];

export function startTutorial(): void {
  let current = 0;
  const overlay = document.createElement('div');
  overlay.id = 'tutorial-overlay';
  overlay.style.position = 'fixed';
  overlay.style.top = '0';
  overlay.style.left = '0';
  overlay.style.right = '0';
  overlay.style.bottom = '0';
  overlay.style.background = 'rgba(0,0,0,0.5)';
  overlay.style.display = 'flex';
  overlay.style.alignItems = 'center';
  overlay.style.justifyContent = 'center';
  overlay.style.zIndex = '1000';
  document.body.appendChild(overlay);

  const box = document.createElement('div');
  box.style.background = '#fff';
  box.style.padding = '1rem';
  box.style.maxWidth = '300px';
  box.style.textAlign = 'center';
  overlay.appendChild(box);

  const text = document.createElement('p');
  box.appendChild(text);

  const progress = document.createElement('p');
  box.appendChild(progress);

  const buttons = document.createElement('div');
  buttons.style.display = 'flex';
  buttons.style.justifyContent = 'space-between';
  buttons.style.marginTop = '0.5rem';
  box.appendChild(buttons);

  const prev = document.createElement('button');
  prev.textContent = t('tutorial_prev');
  prev.addEventListener('click', () => {
    if (current > 0) {
      current--;
      show();
    }
  });
  buttons.appendChild(prev);

  const next = document.createElement('button');
  next.addEventListener('click', () => {
    current++;
    if (current >= steps.length) {
      finish();
    } else {
      show();
    }
  });
  buttons.appendChild(next);

  const style = document.createElement('style');
  style.textContent = '.tutorial-highlight{box-shadow:0 0 0 3px #ff0;}';
  document.head.appendChild(style);

  function show() {
    const step = steps[current];
    text.textContent = t(step.i18nKey);
    progress.textContent = `${t('tutorial_progress')} ${current + 1}/${steps.length}`;
    prev.style.display = current === 0 ? 'none' : 'inline-block';
    next.textContent = current === steps.length - 1 ? t('tutorial_done') : t('tutorial_next');
    document.querySelectorAll('.tutorial-highlight').forEach(el => el.classList.remove('tutorial-highlight'));
    const el = document.querySelector(step.element) as HTMLElement | null;
    if (el) {
      el.classList.add('tutorial-highlight');
    }
  }

  function finish() {
    overlay.remove();
    document.querySelectorAll('.tutorial-highlight').forEach(el => el.classList.remove('tutorial-highlight'));
  }

  show();
}
