export const TOKEN_KEYS = [
  '--bg',
  '--surface',
  '--border',
  '--text',
  '--text-muted',
  '--primary',
  '--primary-contrast',
  '--accent-high',
  '--accent-low',
  '--accent-good',
  '--accent-bad',
] as const;

export type TokenKey = (typeof TOKEN_KEYS)[number];
export type TokenMap = Record<TokenKey, string>;

export interface Preset {
  id: string;
  label: string;
  tokens: TokenMap;
}

export interface ThemeSetting {
  preset: string;
  overrides: Partial<TokenMap>;
}

export const PRESETS: Preset[] = [
  {
    id: 'baseline-warm',
    label: 'Baseline Warm',
    tokens: {
      '--bg': '#f5f0e8',
      '--surface': '#fbfaf6',
      '--border': '#e6ded0',
      '--text': '#1a1a1a',
      '--text-muted': '#7a7368',
      '--primary': '#1f3a3a',
      '--primary-contrast': '#ffffff',
      '--accent-high': '#b8763a',
      '--accent-low': '#3a5878',
      '--accent-good': '#5a7a3a',
      '--accent-bad': '#8a2f2f',
    },
  },
  {
    id: 'cool-slate',
    label: 'Cool Slate',
    tokens: {
      '--bg': '#eef1f5',
      '--surface': '#ffffff',
      '--border': '#d9dee6',
      '--text': '#1c2530',
      '--text-muted': '#5f6b7a',
      '--primary': '#2b3a67',
      '--primary-contrast': '#ffffff',
      '--accent-high': '#c47b2a',
      '--accent-low': '#2f5da8',
      '--accent-good': '#3f8054',
      '--accent-bad': '#a53434',
    },
  },
  {
    id: 'high-contrast',
    label: 'High Contrast',
    tokens: {
      '--bg': '#ffffff',
      '--surface': '#ffffff',
      '--border': '#000000',
      '--text': '#000000',
      '--text-muted': '#333333',
      '--primary': '#000000',
      '--primary-contrast': '#ffffff',
      '--accent-high': '#a35a00',
      '--accent-low': '#003a99',
      '--accent-good': '#005a1a',
      '--accent-bad': '#a30000',
    },
  },
];

export const DEFAULT_PRESET_ID = 'baseline-warm';

export function resolveTheme(setting: ThemeSetting): TokenMap {
  const preset = PRESETS.find((p) => p.id === setting.preset) ?? PRESETS[0];
  return { ...preset.tokens, ...(setting.overrides as TokenMap) };
}

export function applyTheme(el: HTMLElement, setting: ThemeSetting): void {
  const tokens = resolveTheme(setting);
  for (const key of TOKEN_KEYS) {
    el.style.setProperty(key, tokens[key]);
  }
}

const DEFAULT_SETTING: ThemeSetting = { preset: DEFAULT_PRESET_ID, overrides: {} };

export function parseThemeSetting(raw: string): ThemeSetting {
  if (!raw) return { ...DEFAULT_SETTING };
  try {
    const parsed = JSON.parse(raw) as Partial<ThemeSetting>;
    const preset = typeof parsed.preset === 'string' ? parsed.preset : DEFAULT_PRESET_ID;
    const rawOverrides = (parsed.overrides ?? {}) as Record<string, string>;
    const overrides: Partial<TokenMap> = {};
    for (const key of TOKEN_KEYS) {
      if (typeof rawOverrides[key] === 'string') overrides[key] = rawOverrides[key];
    }
    return { preset, overrides };
  } catch {
    return { ...DEFAULT_SETTING };
  }
}

export function serializeThemeSetting(setting: ThemeSetting): string {
  return JSON.stringify(setting);
}
