import settings from '../../settings.json' assert { type: 'json' };
import defaultThemeJson from './themes/default.json' assert { type: 'json' };
import darkThemeJson from './themes/dark.json' assert { type: 'json' };

export interface VisualTheme {
  blockFill: string;
  blockStroke: string;
  blockText: string;
  connection: string;
  highlight: string;
  tooltipBg: string;
  tooltipText: string;
  alignGuide: string;
  blockKinds: Record<string, string>;
}

export const defaultTheme: VisualTheme = defaultThemeJson as VisualTheme;
export const darkTheme: VisualTheme = darkThemeJson as VisualTheme;

// ensure color for literal blocks exists
defaultTheme.blockKinds.Literal = defaultTheme.blockKinds.Literal || '#e1bee7';
darkTheme.blockKinds.Literal = darkTheme.blockKinds.Literal || '#8e24aa';
// ensure color for array blocks exists
defaultTheme.blockKinds.Array = defaultTheme.blockKinds.Array || '#bbdefb';
darkTheme.blockKinds.Array = darkTheme.blockKinds.Array || '#1976d2';
// ensure color for map blocks exists
defaultTheme.blockKinds.Map = defaultTheme.blockKinds.Map || '#c8e6c9';
darkTheme.blockKinds.Map = darkTheme.blockKinds.Map || '#388e3c';
// ensure color for async blocks exists
defaultTheme.blockKinds.Async = defaultTheme.blockKinds.Async || '#b2dfdb';
darkTheme.blockKinds.Async = darkTheme.blockKinds.Async || '#00897b';
// ensure color for logic operator blocks exists
defaultTheme.blockKinds.OpLogic = defaultTheme.blockKinds.OpLogic || '#d1c4e9';
darkTheme.blockKinds.OpLogic = darkTheme.blockKinds.OpLogic || '#7e57c2';
// ensure color for comparison operator blocks exists
defaultTheme.blockKinds.OpComparison = defaultTheme.blockKinds.OpComparison || '#cfd8dc';
darkTheme.blockKinds.OpComparison = darkTheme.blockKinds.OpComparison || '#607d8b';
// ensure color for log blocks exists
defaultTheme.blockKinds.Log = defaultTheme.blockKinds.Log || '#ffe082';
darkTheme.blockKinds.Log = darkTheme.blockKinds.Log || '#ffb300';
// ensure color for arithmetic operator blocks exists
defaultTheme.blockKinds.Operator = defaultTheme.blockKinds.Operator || '#ffcdd2';
darkTheme.blockKinds.Operator = darkTheme.blockKinds.Operator || '#e57373';
// ensure color for sequence blocks exists
defaultTheme.blockKinds.Sequence = defaultTheme.blockKinds.Sequence || '#e6ee9c';
darkTheme.blockKinds.Sequence = darkTheme.blockKinds.Sequence || '#9e9d24';
// ensure color for switch blocks exists
defaultTheme.blockKinds.Switch = defaultTheme.blockKinds.Switch || '#ffe0b2';
darkTheme.blockKinds.Switch = darkTheme.blockKinds.Switch || '#ffb74d';
// ensure color for try blocks exists
defaultTheme.blockKinds.Try = defaultTheme.blockKinds.Try || '#ffccbc';
darkTheme.blockKinds.Try = darkTheme.blockKinds.Try || '#ff8a65';
// ensure color for struct blocks exists
defaultTheme.blockKinds.Struct = defaultTheme.blockKinds.Struct || '#c5cae9';
darkTheme.blockKinds.Struct = darkTheme.blockKinds.Struct || '#5c6bc0';

const themeMap: Record<string, VisualTheme> = {
  default: defaultTheme,
  dark: darkTheme
};

export const availableThemes = Object.keys(themeMap);

type Listener = (theme: VisualTheme) => void;
const listeners = new Set<Listener>();

let currentName = 'default';
let current: VisualTheme = defaultTheme;

export function applyTheme(name: string) {
  const base = themeMap[name] || defaultTheme;
  currentName = name in themeMap ? name : 'default';
  current = { ...base, blockKinds: { ...base.blockKinds } };
  listeners.forEach(l => l(current));
}

export function setTheme(theme: Partial<VisualTheme>) {
  current = {
    ...current,
    ...theme,
    blockKinds: { ...current.blockKinds, ...theme.blockKinds }
  };
  listeners.forEach(l => l(current));
}

export function onThemeChange(listener: Listener) {
  listeners.add(listener);
}

export function getTheme(): VisualTheme {
  return current;
}

export function getThemeName(): string {
  return currentName;
}

const cfg: { visual?: { theme?: string } } = settings as any;
applyTheme(cfg.visual?.theme || 'default');

