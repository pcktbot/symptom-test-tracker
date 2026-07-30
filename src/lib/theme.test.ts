import { describe, it, expect } from 'vitest';
import { PRESETS, DEFAULT_PRESET_ID, resolveTheme, TOKEN_KEYS } from './theme';

describe('theme presets', () => {
  it('exposes the expected preset ids', () => {
    expect(PRESETS.map((p) => p.id)).toEqual(['baseline-warm', 'cool-slate', 'high-contrast']);
  });

  it('every preset defines all token keys', () => {
    for (const preset of PRESETS) {
      for (const key of TOKEN_KEYS) {
        expect(preset.tokens[key], `${preset.id} missing ${key}`).toBeTruthy();
      }
    }
  });

  it('defaults to baseline-warm', () => {
    expect(DEFAULT_PRESET_ID).toBe('baseline-warm');
  });
});

describe('resolveTheme', () => {
  it('returns preset tokens when there are no overrides', () => {
    const resolved = resolveTheme({ preset: 'baseline-warm', overrides: {} });
    const preset = PRESETS.find((p) => p.id === 'baseline-warm')!;
    expect(resolved).toEqual(preset.tokens);
  });

  it('applies overrides on top of preset tokens', () => {
    const resolved = resolveTheme({
      preset: 'baseline-warm',
      overrides: { '--primary': '#ff0000' },
    });
    expect(resolved['--primary']).toBe('#ff0000');
    expect(resolved['--bg']).toBe(PRESETS[0].tokens['--bg']);
  });

  it('falls back to default preset when preset id is unknown', () => {
    const resolved = resolveTheme({ preset: 'nope', overrides: {} });
    expect(resolved['--bg']).toBe(PRESETS[0].tokens['--bg']);
  });
});
