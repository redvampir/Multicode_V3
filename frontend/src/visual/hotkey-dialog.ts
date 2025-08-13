import type { HotkeyMap } from './hotkeys.ts';

export function createHotkeyDialog(hotkeys: HotkeyMap): HTMLDialogElement {
  const dialog = document.createElement('dialog');
  dialog.id = 'hotkey-help';

  const title = document.createElement('h2');
  title.id = 'hotkey-help-title';
  title.textContent = 'Hotkeys';
  dialog.appendChild(title);
  dialog.setAttribute('aria-labelledby', title.id);

  const list = document.createElement('dl');
  for (const [name, combo] of Object.entries(hotkeys)) {
    const dt = document.createElement('dt');
    const kbd = document.createElement('kbd');
    kbd.textContent = combo;
    dt.appendChild(kbd);

    const dd = document.createElement('dd');
    dd.textContent = name;
    list.appendChild(dt);
    list.appendChild(dd);
  }
  dialog.appendChild(list);

  const closeBtn = document.createElement('button');
  closeBtn.textContent = 'Close';
  closeBtn.addEventListener('click', () => dialog.close());
  dialog.appendChild(closeBtn);

  document.body.appendChild(dialog);
  return dialog;
}
