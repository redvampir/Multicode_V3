export type PanelType = 'editor' | 'canvas' | 'terminal';

import Ajv from 'ajv';
import addFormats from 'ajv-formats';
import schema from '../editor/meta.schema.json' with { type: 'json' };

const ajv = new Ajv({ allErrors: true });
addFormats(ajv);
const validateMeta = ajv.compile(schema);

interface Panel {
  id: string;
  type: PanelType;
  element: HTMLElement;
  metadata: Record<string, any>;
  onMeta?: (meta: Record<string, any>) => void;
  setMetadata: (meta: Record<string, any>) => void;
}

interface SplitConfig {
  orientation: 'horizontal' | 'vertical';
  panels: { type: PanelType; size: number }[];
}

const STORAGE_KEY = 'split-view-config';

function loadConfig(): SplitConfig | null {
  try {
    if (typeof localStorage !== 'undefined') {
      const raw = localStorage.getItem(STORAGE_KEY);
      if (raw) return JSON.parse(raw);
    }
  } catch (e) {
    console.warn('loadConfig failed', e);
  }
  return null;
}

function saveConfig(cfg: SplitConfig) {
  try {
    if (typeof localStorage !== 'undefined') {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(cfg));
    }
  } catch (e) {
    console.warn('saveConfig failed', e);
  }
}

export class SplitManager {
  container: HTMLElement;
  orientation: 'row' | 'column';
  panels: Panel[] = [];
  metadata: Record<string, any> = {};

  constructor(container: HTMLElement, config?: SplitConfig) {
    this.container = container;
    const cfg = config || loadConfig() || { orientation: 'horizontal', panels: [] };
    this.orientation = cfg.orientation === 'vertical' ? 'column' : 'row';
    container.style.display = 'flex';
    container.style.flexDirection = this.orientation;

    if (cfg.panels.length) {
      cfg.panels.forEach(p => this.addPanel(p.type, p.size));
    }
  }

  addPanel(type: PanelType, size?: number): Panel {
    if (this.panels.length >= 4) throw new Error('Maximum of 4 panels supported');

    const el = document.createElement('div');
    el.className = 'split-panel';
    el.style.flex = size ? `0 0 ${size}%` : '1';
    el.style.position = 'relative';
    el.dataset.type = type;

    const panel: Panel = {
      id: Math.random().toString(36).slice(2),
      type,
      element: el,
      metadata: {},
      setMetadata: (meta) => {
        const vm = meta['@VISUAL_META'];
        if (vm && !validateMeta(vm)) {
          console.warn('invalid @VISUAL_META', validateMeta.errors);
          return;
        }
        panel.metadata = { ...panel.metadata, ...meta };
        this.updateMetadata(meta, panel.id);
      }
    };

    this.panels.push(panel);
    this.container.appendChild(el);

    if (this.panels.length > 1) {
      const resizer = document.createElement('div');
      resizer.className = 'split-resizer';
      resizer.style.flex = '0 0 4px';
      resizer.style.background = 'transparent';
      resizer.style.cursor = this.orientation === 'row' ? 'col-resize' : 'row-resize';
      resizer.addEventListener('mousedown', e => this.startDrag(e, panel));
      this.container.insertBefore(resizer, el);
    }

    this.save();
    return panel;
  }

  private startDrag(e: MouseEvent, panel: Panel) {
    e.preventDefault();
    const index = this.panels.indexOf(panel);
    const prev = this.panels[index - 1];
    const start = this.orientation === 'row' ? e.clientX : e.clientY;
    const prevRect = prev.element.getBoundingClientRect();
    const panelRect = panel.element.getBoundingClientRect();
    const prevSize = this.orientation === 'row' ? prevRect.width : prevRect.height;
    const panelSize = this.orientation === 'row' ? panelRect.width : panelRect.height;
    const total = prevSize + panelSize;

    const move = (ev: MouseEvent) => {
      const current = this.orientation === 'row' ? ev.clientX : ev.clientY;
      const delta = current - start;
      const prevPercent = ((prevSize + delta) / total) * 100;
      const panelPercent = ((panelSize - delta) / total) * 100;
      prev.element.style.flex = `0 0 ${prevPercent}%`;
      panel.element.style.flex = `0 0 ${panelPercent}%`;
    };

    const up = () => {
      document.removeEventListener('mousemove', move);
      document.removeEventListener('mouseup', up);
      this.save();
    };

    document.addEventListener('mousemove', move);
    document.addEventListener('mouseup', up);
  }

  private updateMetadata(meta: Record<string, any>, from?: string) {
    this.metadata = { ...this.metadata, ...meta };
    this.panels.forEach(p => {
      if (p.id !== from) {
        p.metadata = { ...p.metadata, ...meta };
        p.onMeta?.(meta);
      }
    });
  }

  private save() {
    const cfg: SplitConfig = {
      orientation: this.orientation === 'column' ? 'vertical' : 'horizontal',
      panels: this.panels.map(p => {
        const basis = p.element.style.flexBasis;
        const size = basis ? parseFloat(basis) : 100 / this.panels.length;
        return { type: p.type, size };
      })
    };
    saveConfig(cfg);
  }

  static load(container: HTMLElement) {
    const cfg = loadConfig();
    return new SplitManager(container, cfg || undefined);
  }
}

