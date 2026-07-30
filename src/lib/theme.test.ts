import { describe, it, expect } from 'vitest';
import { PRESETS, DEFAULT_PRESET_ID, resolveTheme, TOKEN_KEYS, ThemeSetting, applyTheme, parseThemeSetting, serializeThemeSetting } from './theme';

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

describe('applyTheme', () => {
  it('sets every token as an inline style on the given element', () => {
    const el = document.createElement('div');
    applyTheme(el, { preset: 'baseline-warm', overrides: {} });
    expect(el.style.getPropertyValue('--primary')).toBe('#1f3a3a');
    expect(el.style.getPropertyValue('--bg')).toBe('#f5f0e8');
  });

  it('applies overrides', () => {
    const el = document.createElement('div');
    applyTheme(el, { preset: 'baseline-warm', overrides: { '--primary': '#ff0000' } });
    expect(el.style.getPropertyValue('--primary')).toBe('#ff0000');
  });
});

describe('parseThemeSetting', () => {
  it('returns default when input is empty', () => {
    expect(parseThemeSetting('')).toEqual({ preset: 'baseline-warm', overrides: {} });
  });

  it('returns default when input is invalid JSON', () => {
    expect(parseThemeSetting('not json')).toEqual({ preset: 'baseline-warm', overrides: {} });
  });

  it('returns parsed setting', () => {
    const s = JSON.stringify({ preset: 'cool-slate', overrides: { '--primary': '#111' } });
    expect(parseThemeSetting(s)).toEqual({ preset: 'cool-slate', overrides: { '--primary': '#111' } });
  });

  it('drops unknown override keys', () => {
    const s = JSON.stringify({ preset: 'baseline-warm', overrides: { '--primary': '#111', '--bogus': '#000' } });
    expect(parseThemeSetting(s).overrides).toEqual({ '--primary': '#111' });
  });
});

describe('serializeThemeSetting', () => {
  it('round-trips through parseThemeSetting', () => {
    const s: ThemeSetting = { preset: 'cool-slate', overrides: { '--bg': '#000' } };
    expect(parseThemeSetting(serializeThemeSetting(s))).toEqual(s);
  });
});
