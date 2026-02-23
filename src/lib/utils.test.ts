import { describe, it, expect, vi, beforeEach } from 'vitest';
import { todayString, formatDate, flagLabel, flagClass, LAB_PANELS, getMergedPanels } from './utils';
import { getCustomLabTests } from './db';

vi.mock('./db', () => ({
  getCustomLabTests: vi.fn().mockResolvedValue([]),
}));

const mockGetCustomLabTests = vi.mocked(getCustomLabTests);

describe('todayString', () => {
  it('returns current date in YYYY-MM-DD format', () => {
    const result = todayString();
    expect(result).toMatch(/^\d{4}-\d{2}-\d{2}$/);
  });

  it('matches Date ISO output', () => {
    const expected = new Date().toISOString().slice(0, 10);
    expect(todayString()).toBe(expected);
  });
});

describe('formatDate', () => {
  it('formats a date string to readable US format', () => {
    const result = formatDate('2025-01-15');
    expect(result).toBe('Jan 15, 2025');
  });

  it('handles single-digit days', () => {
    const result = formatDate('2025-03-05');
    expect(result).toBe('Mar 5, 2025');
  });

  it('handles December correctly', () => {
    const result = formatDate('2024-12-31');
    expect(result).toBe('Dec 31, 2024');
  });

  it('handles beginning of year', () => {
    const result = formatDate('2026-01-01');
    expect(result).toBe('Jan 1, 2026');
  });
});

describe('flagLabel', () => {
  it('returns "Critical High" for HH', () => {
    expect(flagLabel('HH')).toBe('Critical High');
  });

  it('returns "High" for H', () => {
    expect(flagLabel('H')).toBe('High');
  });

  it('returns "Critical Low" for LL', () => {
    expect(flagLabel('LL')).toBe('Critical Low');
  });

  it('returns "Low" for L', () => {
    expect(flagLabel('L')).toBe('Low');
  });

  it('returns "Normal" for N', () => {
    expect(flagLabel('N')).toBe('Normal');
  });

  it('returns "Normal" for empty string', () => {
    expect(flagLabel('')).toBe('Normal');
  });

  it('returns "Normal" for unknown values', () => {
    expect(flagLabel('X')).toBe('Normal');
  });
});

describe('flagClass', () => {
  it('returns flag-critical for HH', () => {
    expect(flagClass('HH')).toBe('flag-critical');
  });

  it('returns flag-critical for LL', () => {
    expect(flagClass('LL')).toBe('flag-critical');
  });

  it('returns flag-high for H', () => {
    expect(flagClass('H')).toBe('flag-high');
  });

  it('returns flag-low for L', () => {
    expect(flagClass('L')).toBe('flag-low');
  });

  it('returns flag-normal for N', () => {
    expect(flagClass('N')).toBe('flag-normal');
  });

  it('returns flag-normal for unknown values', () => {
    expect(flagClass('whatever')).toBe('flag-normal');
  });
});

describe('LAB_PANELS', () => {
  it('contains expected panel names', () => {
    const names = LAB_PANELS.map(p => p.name);
    expect(names).toContain('CBC');
    expect(names).toContain('Lipids');
    expect(names).toContain('Inflammation');
    expect(names).toContain('Lupus/APS Markers');
    expect(names).toContain('Metabolic / BMP');
    expect(names).toContain('Thyroid');
    expect(names).toContain('Coagulation');
  });

  it('has 7 panels', () => {
    expect(LAB_PANELS).toHaveLength(7);
  });

  it('CBC panel has 14 tests', () => {
    const cbc = LAB_PANELS.find(p => p.name === 'CBC');
    expect(cbc?.tests).toHaveLength(14);
  });

  it('each test has name and unit', () => {
    for (const panel of LAB_PANELS) {
      for (const test of panel.tests) {
        expect(test.name).toBeTruthy();
        expect(test).toHaveProperty('unit');
        expect(test).toHaveProperty('ref_low');
        expect(test).toHaveProperty('ref_high');
      }
    }
  });

  it('text_only tests exist in Lupus/APS panel', () => {
    const lupus = LAB_PANELS.find(p => p.name === 'Lupus/APS Markers');
    const textOnly = lupus?.tests.filter(t => t.text_only);
    expect(textOnly?.length).toBeGreaterThanOrEqual(1);
    expect(textOnly?.map(t => t.name)).toContain('ANA');
    expect(textOnly?.map(t => t.name)).toContain('Lupus Anticoagulant');
    expect(textOnly?.map(t => t.name)).toContain('Direct Coombs');
  });
});

describe('getMergedPanels', () => {
  beforeEach(() => {
    mockGetCustomLabTests.mockReset();
  });

  it('returns built-in panels when no custom tests exist', async () => {
    mockGetCustomLabTests.mockResolvedValue([]);

    const panels = await getMergedPanels();
    expect(panels).toHaveLength(LAB_PANELS.length);
    expect(panels.map(p => p.name)).toEqual(LAB_PANELS.map(p => p.name));
  });

  it('adds custom tests to existing panels', async () => {
    mockGetCustomLabTests.mockResolvedValue([
      {
        id: 1,
        name: 'Custom Test',
        panel: 'CBC',
        unit: 'mg/dL',
        ref_low: 1.0,
        ref_high: 10.0,
        text_only: false,
        description: '',
        clinical: '',
      },
    ]);

    const panels = await getMergedPanels();
    const cbc = panels.find(p => p.name === 'CBC');
    expect(cbc?.tests.some(t => t.name === 'Custom Test')).toBe(true);
  });

  it('creates new panel for custom tests with new panel name', async () => {
    mockGetCustomLabTests.mockResolvedValue([
      {
        id: 2,
        name: 'Special Marker',
        panel: 'Special Panel',
        unit: 'U/L',
        ref_low: 0,
        ref_high: 50,
        text_only: false,
        description: '',
        clinical: '',
      },
    ]);

    const panels = await getMergedPanels();
    const special = panels.find(p => p.name === 'Special Panel');
    expect(special).toBeDefined();
    expect(special?.tests).toHaveLength(1);
    expect(special?.tests[0].name).toBe('Special Marker');
  });

  it('does not duplicate built-in tests when custom has same name', async () => {
    mockGetCustomLabTests.mockResolvedValue([
      {
        id: 3,
        name: 'WBC',
        panel: 'CBC',
        unit: 'K/uL',
        ref_low: 4.5,
        ref_high: 11.0,
        text_only: false,
        description: '',
        clinical: '',
      },
    ]);

    const panels = await getMergedPanels();
    const cbc = panels.find(p => p.name === 'CBC');
    const wbcCount = cbc?.tests.filter(t => t.name === 'WBC').length;
    expect(wbcCount).toBe(1);
  });
});
