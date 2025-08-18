/* @vitest-environment jsdom */

import { describe, it, expect, vi } from 'vitest';
import { initPluginManager } from './manager';

describe('initPluginManager', () => {
  it('alerts when plugin list fails to load', async () => {
    const container = document.createElement('div');
    const fetchMock = vi.fn().mockResolvedValue({ ok: false });
    const alertMock = vi.fn();
    const originalFetch = globalThis.fetch;
    const originalAlert = (globalThis as any).alert;
    // @ts-ignore
    globalThis.fetch = fetchMock;
    // @ts-ignore
    globalThis.alert = alertMock;

    await initPluginManager(container);

    expect(alertMock).toHaveBeenCalled();

    globalThis.fetch = originalFetch;
    // @ts-ignore
    globalThis.alert = originalAlert;
  });

  it('alerts when plugin update fails', async () => {
    const container = document.createElement('div');
    const plugins = [{ name: 'test', version: '1.0.0', enabled: false }];
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce({ ok: true, json: async () => plugins })
      .mockResolvedValueOnce({ ok: false });
    const alertMock = vi.fn();
    const originalFetch = globalThis.fetch;
    const originalAlert = (globalThis as any).alert;
    // @ts-ignore
    globalThis.fetch = fetchMock;
    // @ts-ignore
    globalThis.alert = alertMock;

    await initPluginManager(container);

    const checkbox = container.querySelector('input') as HTMLInputElement;
    checkbox.checked = true;
    checkbox.dispatchEvent(new Event('change'));
    await Promise.resolve();

    expect(alertMock).toHaveBeenCalled();
    expect(checkbox.checked).toBe(false);

    globalThis.fetch = originalFetch;
    // @ts-ignore
    globalThis.alert = originalAlert;
  });
});
