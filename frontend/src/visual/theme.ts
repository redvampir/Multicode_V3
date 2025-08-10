import settings from '../../settings.json' assert { type: 'json' };
import defaultThemeJson from './themes/default.json' assert { type: 'json' };

export interface VisualTheme {
  blockFill: string;
  blockStroke: string;
  blockText: string;
  connection: string;
  highlight: string;
  tooltipBg: string;
  tooltipText: string;
  blockKinds: Record<string, string>;
}

export const defaultTheme: VisualTheme = defaultThemeJson as VisualTheme;

const themes: Record<string, VisualTheme> = {
  default: defaultTheme
};

const cfg: { visual?: { theme?: string } } = settings as any;
const themeName = cfg.visual?.theme || 'default';
const base = themes[themeName] || defaultTheme;

let current: VisualTheme = {
  ...base,
  blockKinds: { ...base.blockKinds }
};

export function setTheme(theme: Partial<VisualTheme>) {
  current = {
    ...current,
    ...theme,
    blockKinds: { ...current.blockKinds, ...theme.blockKinds }
  };
}

export function getTheme(): VisualTheme {
  return current;
}

