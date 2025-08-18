import type { VizPlugin, PluginRegistry } from './index.ts';

/**
 * Load all visual plugins from the `plugins/` directory.
 *
 * Each plugin should export a `register` function that accepts a registry
 * object.  The function is invoked immediately when the plugin is loaded.
 */
export function loadPlugins(registry: PluginRegistry) {
  const modules = import.meta.glob<Promise<VizPlugin | { register: Function }>>(
    '../../../plugins/*/index.{ts,js}',
    { eager: true }
  );
  for (const path in modules) {
    const mod: any = modules[path];
    if (mod && typeof mod.register === 'function') {
      mod.register(registry);
    }
  }
}
