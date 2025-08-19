import { describe, it, expect } from 'vitest';
import { EventBus } from '../eventBus.js';

describe('EventBus', () => {
  it('emits events to listeners', () => {
    const bus = new EventBus();
    let called = false;
    bus.on('test', () => { called = true; });
    bus.emit('test');
    expect(called).toBe(true);
  });
  it('ignores invalid inputs', () => {
    const bus = new EventBus();
    bus.on(null, null);
    bus.emit(null);
    expect(bus.events.size).toBe(0);
  });
});
