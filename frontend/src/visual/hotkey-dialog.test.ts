// @vitest-environment jsdom
import { describe, it, expect, vi } from 'vitest';
vi.mock('../../scripts/format.js', () => ({ formatCurrentFile: vi.fn() }));
import { createHotkeyDialog } from './hotkey-dialog.ts';
import { hotkeys } from './hotkeys.ts';

describe('hotkey dialog', () => {
  it('includes palette shortcut', () => {
    const dialog = createHotkeyDialog(hotkeys);
    const text = dialog.textContent || '';
    expect(text).toContain('Ctrl+P');
    expect(text).toContain('Space Space');
  });
});
