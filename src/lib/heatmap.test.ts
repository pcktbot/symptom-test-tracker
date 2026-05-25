import { describe, it, expect } from 'vitest';
import {
  scoreToColor,
  scoreToLabel,
  remapScore,
  parseTags,
  serializeTags,
  isValidDate,
  DEFAULT_CONFIG,
} from './heatmap';

describe('scoreToColor', () => {
  it('returns color for score 1 (worst)', () => {
    expect(scoreToColor(1, DEFAULT_CONFIG)).toBe('#c0392b');
  });
  it('returns color for score 5 (best)', () => {
    expect(scoreToColor(5, DEFAULT_CONFIG)).toBe('#1e8449');
  });
  it('returns empty string for score 0', () => {
    expect(scoreToColor(0, DEFAULT_CONFIG)).toBe('');
  });
  it('returns empty string for score above steps', () => {
    expect(scoreToColor(6, DEFAULT_CONFIG)).toBe('');
  });
});

describe('scoreToLabel', () => {
  it('returns label for score 1', () => {
    expect(scoreToLabel(1, DEFAULT_CONFIG)).toBe('Worst');
  });
  it('returns label for score 3', () => {
    expect(scoreToLabel(3, DEFAULT_CONFIG)).toBe('Okay');
  });
  it('returns empty string for out-of-range score', () => {
    expect(scoreToLabel(0, DEFAULT_CONFIG)).toBe('');
  });
});

describe('remapScore', () => {
  it('maps score 1 to 1 regardless of step counts', () => {
    expect(remapScore(1, 5, 3)).toBe(1);
  });
  it('maps max score to max score', () => {
    expect(remapScore(5, 5, 3)).toBe(3);
  });
  it('maps midpoint proportionally (5→3)', () => {
    expect(remapScore(3, 5, 3)).toBe(2);
  });
  it('maps 1-step to 5-step: min stays 1', () => {
    expect(remapScore(1, 3, 5)).toBe(1);
  });
  it('maps midpoint proportionally (3→5)', () => {
    expect(remapScore(2, 3, 5)).toBe(3);
  });
  it('maps max (3→5): becomes 5', () => {
    expect(remapScore(3, 3, 5)).toBe(5);
  });
  it('identity: same step count returns same score', () => {
    expect(remapScore(3, 5, 5)).toBe(3);
    expect(remapScore(1, 5, 5)).toBe(1);
  });
  it('clamps to [1, newSteps]', () => {
    expect(remapScore(5, 5, 2)).toBeGreaterThanOrEqual(1);
    expect(remapScore(5, 5, 2)).toBeLessThanOrEqual(2);
  });
});

describe('parseTags', () => {
  it('parses a JSON array string', () => {
    expect(parseTags('["fatigue","brain fog"]')).toEqual(['fatigue', 'brain fog']);
  });
  it('returns an array as-is', () => {
    expect(parseTags(['fatigue'])).toEqual(['fatigue']);
  });
  it('returns empty array for empty string', () => {
    expect(parseTags('')).toEqual([]);
  });
  it('returns empty array for empty JSON array', () => {
    expect(parseTags('[]')).toEqual([]);
  });
});

describe('serializeTags', () => {
  it('serializes to JSON string', () => {
    expect(serializeTags(['fatigue', 'brain fog'])).toBe('["fatigue","brain fog"]');
  });
  it('round-trips with parseTags', () => {
    const tags = ['fatigue', 'brain fog'];
    expect(parseTags(serializeTags(tags))).toEqual(tags);
  });
});

describe('isValidDate', () => {
  it('accepts Jan 31', () => {
    expect(isValidDate(2026, 1, 31)).toBe(true);
  });
  it('accepts Feb 28', () => {
    expect(isValidDate(2026, 2, 28)).toBe(true);
  });
  it('rejects Feb 30', () => {
    expect(isValidDate(2026, 2, 30)).toBe(false);
  });
  it('rejects April 31', () => {
    expect(isValidDate(2026, 4, 31)).toBe(false);
  });
  it('accepts Feb 29 on a leap year', () => {
    expect(isValidDate(2024, 2, 29)).toBe(true);
  });
  it('rejects Feb 29 on a non-leap year', () => {
    expect(isValidDate(2026, 2, 29)).toBe(false);
  });
});
