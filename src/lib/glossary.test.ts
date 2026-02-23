import { describe, it, expect, vi, beforeEach } from 'vitest';
import { GLOSSARY, getGlossaryEntry, getGlossaryByPanel, getMergedGlossary } from './glossary';
import { getCustomLabTests } from './db';

vi.mock('./db', () => ({
  getCustomLabTests: vi.fn().mockResolvedValue([]),
}));

const mockGetCustomLabTests = vi.mocked(getCustomLabTests);

describe('GLOSSARY', () => {
  it('contains entries for all 7 panels', () => {
    const panels = new Set(GLOSSARY.map(e => e.panel));
    expect(panels.size).toBe(7);
    expect(panels).toContain('CBC');
    expect(panels).toContain('Lipids');
    expect(panels).toContain('Inflammation');
    expect(panels).toContain('Lupus/APS Markers');
    expect(panels).toContain('Metabolic / BMP');
    expect(panels).toContain('Thyroid');
    expect(panels).toContain('Coagulation');
  });

  it('every entry has name, panel, description, and clinical fields', () => {
    for (const entry of GLOSSARY) {
      expect(entry.name).toBeTruthy();
      expect(entry.panel).toBeTruthy();
      expect(entry.description).toBeTruthy();
      expect(entry.clinical).toBeTruthy();
    }
  });

  it('has no duplicate names', () => {
    const names = GLOSSARY.map(e => e.name);
    const unique = new Set(names);
    expect(unique.size).toBe(names.length);
  });
});

describe('getGlossaryEntry', () => {
  it('returns the entry for a known test name', () => {
    const entry = getGlossaryEntry('WBC');
    expect(entry).toBeDefined();
    expect(entry?.name).toBe('WBC');
    expect(entry?.panel).toBe('CBC');
    expect(entry?.description).toContain('White blood cell');
  });

  it('returns undefined for unknown test name', () => {
    expect(getGlossaryEntry('NonexistentTest')).toBeUndefined();
  });

  it('is case-sensitive', () => {
    expect(getGlossaryEntry('wbc')).toBeUndefined();
  });

  it('finds tests from different panels', () => {
    expect(getGlossaryEntry('TSH')?.panel).toBe('Thyroid');
    expect(getGlossaryEntry('CRP')?.panel).toBe('Inflammation');
    expect(getGlossaryEntry('INR')?.panel).toBe('Coagulation');
    expect(getGlossaryEntry('Anti-dsDNA')?.panel).toBe('Lupus/APS Markers');
  });
});

describe('getGlossaryByPanel', () => {
  it('groups entries by panel name', () => {
    const groups = getGlossaryByPanel();
    expect(Object.keys(groups)).toHaveLength(7);
  });

  it('CBC group contains all CBC entries', () => {
    const groups = getGlossaryByPanel();
    const cbcEntries = GLOSSARY.filter(e => e.panel === 'CBC');
    expect(groups['CBC']).toHaveLength(cbcEntries.length);
  });

  it('each entry in a group belongs to that panel', () => {
    const groups = getGlossaryByPanel();
    for (const [panel, entries] of Object.entries(groups)) {
      for (const entry of entries) {
        expect(entry.panel).toBe(panel);
      }
    }
  });

  it('total entries across groups equals GLOSSARY length', () => {
    const groups = getGlossaryByPanel();
    const total = Object.values(groups).reduce((sum, arr) => sum + arr.length, 0);
    expect(total).toBe(GLOSSARY.length);
  });
});

describe('getMergedGlossary', () => {
  beforeEach(() => {
    mockGetCustomLabTests.mockReset();
  });

  it('returns built-in glossary when no custom tests exist', async () => {
    mockGetCustomLabTests.mockResolvedValue([]);

    const groups = await getMergedGlossary();
    expect(Object.keys(groups)).toHaveLength(7);
  });

  it('adds custom test with description to glossary', async () => {
    mockGetCustomLabTests.mockResolvedValue([
      {
        id: 1,
        name: 'Custom Marker',
        panel: 'CBC',
        unit: 'mg/dL',
        ref_low: 1.0,
        ref_high: 10.0,
        text_only: false,
        description: 'A custom marker description',
        clinical: 'Clinical significance here',
      },
    ]);

    const groups = await getMergedGlossary();
    const cbcEntry = groups['CBC']?.find(e => e.name === 'Custom Marker');
    expect(cbcEntry).toBeDefined();
    expect(cbcEntry?.description).toBe('A custom marker description');
  });

  it('skips custom tests with no description or clinical info', async () => {
    mockGetCustomLabTests.mockResolvedValue([
      {
        id: 2,
        name: 'Empty Test',
        panel: 'Lipids',
        unit: 'mg/dL',
        ref_low: 0,
        ref_high: 100,
        text_only: false,
        description: '',
        clinical: '',
      },
    ]);

    const groups = await getMergedGlossary();
    const found = groups['Lipids']?.find(e => e.name === 'Empty Test');
    expect(found).toBeUndefined();
  });

  it('creates new panel group for custom test in new panel', async () => {
    mockGetCustomLabTests.mockResolvedValue([
      {
        id: 3,
        name: 'New Panel Test',
        panel: 'Hormones',
        unit: 'pg/mL',
        ref_low: 10,
        ref_high: 100,
        text_only: false,
        description: 'Hormone test',
        clinical: 'Tracks hormone levels',
      },
    ]);

    const groups = await getMergedGlossary();
    expect(groups['Hormones']).toBeDefined();
    expect(groups['Hormones']).toHaveLength(1);
  });
});
