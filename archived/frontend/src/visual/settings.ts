import settings from '../../settings.json' assert { type: 'json' };

export interface VisualSettings {
  gridSize: number;
  showGrid: boolean;
  [key: string]: any;
}

const cfg = (settings as any).visual || {};

export const visualSettings: VisualSettings = {
  ...cfg,
  gridSize: cfg.gridSize || 20,
  showGrid: cfg.showGrid ?? false
};

export const GRID_SIZE = visualSettings.gridSize;
