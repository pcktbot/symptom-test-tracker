import { describe, it, expect } from 'vitest';
import { DEMO_APPOINTMENTS, DEMO_PROVIDERS, upcomingCare } from './demo';

describe('demo data', () => {
  it('exposes appointments with expected shape', () => {
    expect(DEMO_APPOINTMENTS.length).toBeGreaterThan(0);
    for (const a of DEMO_APPOINTMENTS) {
      expect(a.date).toMatch(/^\d{4}-\d{2}-\d{2}$/);
      expect(typeof a.title).toBe('string');
      expect(typeof a.location).toBe('string');
    }
  });

  it('exposes providers with initials derived from name', () => {
    expect(DEMO_PROVIDERS.length).toBeGreaterThan(0);
    for (const p of DEMO_PROVIDERS) {
      expect(p.initials).toHaveLength(2);
      expect(typeof p.specialty).toBe('string');
    }
  });

  it('upcomingCare returns the first N appointments sorted by date', () => {
    const items = upcomingCare(3);
    expect(items).toHaveLength(3);
    for (let i = 1; i < items.length; i++) {
      expect(items[i - 1].date <= items[i].date).toBe(true);
    }
  });
});
