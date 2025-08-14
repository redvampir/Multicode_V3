// Simple shared event bus for frontend modules.
// Events:
// - blockSelected { id: string }
// - blockInfoRequest { id: string }
// - blockInfo { id: string, kind: string, color: string, thumbnail?: string }
// - metaUpdated { id: string, x?: number, y?: number }
// - edgeNotFound { from: string, to: string }
// - blockCreated { id: string, kind: string }
// - blockRemoved { id: string }
// - lintReported { errors: Record<string,string> }
// - testResult { id: string, success: boolean }
// - edgeSelected { from: string, to: string }
// - blocksReordered { ids: string[] }
// - refreshText { updates: Record<string,string> }
const listeners = new Map();

/**
 * Subscribe to event.
 * @param {string} event
 * @param {(data:any)=>void} handler
 * @returns {() => void}
 */
export function on(event, handler) {
  if (!listeners.has(event)) listeners.set(event, new Set());
  listeners.get(event).add(handler);
  return () => listeners.get(event)?.delete(handler);
}

/**
 * Emit event to subscribers.
 * @param {string} event
 * @param {any} data
 */
export function emit(event, data) {
  const handlers = listeners.get(event);
  if (handlers) {
    handlers.forEach(fn => {
      try {
        fn(data);
      } catch (e) {
        console.error('event handler error', e);
      }
    });
  }
}

