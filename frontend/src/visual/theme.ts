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

export const defaultTheme: VisualTheme = {
  blockFill: '#fff',
  blockStroke: '#333',
  blockText: '#000',
  connection: '#000',
  highlight: '#ffcccc',
  tooltipBg: '#333',
  tooltipText: '#fff',
  blockKinds: {
    Function: '#e0f7fa',
    Variable: '#f1f8e9',
    Condition: '#fff9c4',
    Loop: '#fce4ec',
  }
};

let current: VisualTheme = {
  ...defaultTheme,
  blockKinds: { ...defaultTheme.blockKinds }
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

