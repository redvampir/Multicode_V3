/** Simple event bus with input validation. */
export class EventBus {
  constructor() {
    this.events = new Map();
  }

  /**
   * Subscribe to an event.
   * @param {unknown} event
   * @param {unknown} listener
   */
  on(event, listener) {
    if (typeof event !== 'string' || typeof listener !== 'function') return;
    if (!this.events.has(event)) this.events.set(event, new Set());
    this.events.get(event).add(listener);
  }

  /**
   * Unsubscribe from an event.
   * @param {unknown} event
   * @param {unknown} listener
   */
  off(event, listener) {
    if (typeof event !== 'string' || typeof listener !== 'function') return;
    this.events.get(event)?.delete(listener);
  }

  /**
   * Emit an event.
   * @param {unknown} event
   * @param  {...unknown} args
   */
  emit(event, ...args) {
    if (typeof event !== 'string') return;
    const listeners = this.events.get(event);
    if (!listeners) return;
    for (const l of listeners) {
      try {
        l(...args);
      } catch {
        // ignore listener errors to avoid breaking bus
      }
    }
  }
}
