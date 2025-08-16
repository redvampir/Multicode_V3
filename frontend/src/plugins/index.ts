export interface PluginRegistry {
  registerBlock: (kind: string, ctor: unknown) => void;
  unregisterBlock?: (kind: string) => void;
  [key: string]: unknown;
}

export interface VizPlugin {
  register(registry: PluginRegistry): void;
}
