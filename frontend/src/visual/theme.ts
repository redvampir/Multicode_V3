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

