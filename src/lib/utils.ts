import type { Flag, PanelDefinition, CustomLabTest } from './types';
import { getCustomLabTests } from './db';

export function todayString(): string {
  return new Date().toISOString().slice(0, 10);
}

export function formatDate(dateStr: string): string {
  const d = new Date(dateStr + 'T00:00:00');
  return d.toLocaleDateString('en-US', { month: 'short', day: 'numeric', year: 'numeric' });
}

export function flagLabel(flag: string): string {
  switch (flag) {
    case 'HH': return 'Critical High';
    case 'H': return 'High';
    case 'LL': return 'Critical Low';
    case 'L': return 'Low';
    default: return 'Normal';
  }
}

export function flagClass(flag: string): string {
  switch (flag) {
    case 'HH':
    case 'LL':
      return 'flag-critical';
    case 'H':
      return 'flag-high';
    case 'L':
      return 'flag-low';
    default:
      return 'flag-normal';
  }
}

export const LAB_PANELS: PanelDefinition[] = [
  {
    name: 'CBC',
    tests: [
      { name: 'WBC', unit: 'K/uL', ref_low: 4.5, ref_high: 11.0 },
      { name: 'RBC', unit: 'M/uL', ref_low: 4.0, ref_high: 5.5 },
      { name: 'Hemoglobin', unit: 'g/dL', ref_low: 12.0, ref_high: 16.0 },
      { name: 'Hematocrit', unit: '%', ref_low: 36, ref_high: 46 },
      { name: 'MCV', unit: 'fL', ref_low: 80, ref_high: 100 },
      { name: 'MCH', unit: 'pg', ref_low: 27, ref_high: 33 },
      { name: 'MCHC', unit: 'g/dL', ref_low: 32, ref_high: 36 },
      { name: 'RDW', unit: '%', ref_low: 11.5, ref_high: 14.5 },
      { name: 'Platelets', unit: 'K/uL', ref_low: 150, ref_high: 400 },
      { name: 'Neutrophils', unit: '%', ref_low: 40, ref_high: 70 },
      { name: 'Lymphocytes', unit: '%', ref_low: 20, ref_high: 40 },
      { name: 'Monocytes', unit: '%', ref_low: 2, ref_high: 8 },
      { name: 'Eosinophils', unit: '%', ref_low: 1, ref_high: 4 },
      { name: 'Basophils', unit: '%', ref_low: 0, ref_high: 1 },
    ],
  },
  {
    name: 'Lipids',
    tests: [
      { name: 'Total Cholesterol', unit: 'mg/dL', ref_low: null, ref_high: 200 },
      { name: 'LDL', unit: 'mg/dL', ref_low: null, ref_high: 100 },
      { name: 'HDL', unit: 'mg/dL', ref_low: 40, ref_high: null },
      { name: 'Triglycerides', unit: 'mg/dL', ref_low: null, ref_high: 150 },
      { name: 'VLDL', unit: 'mg/dL', ref_low: 5, ref_high: 40 },
      { name: 'Non-HDL', unit: 'mg/dL', ref_low: null, ref_high: 130 },
    ],
  },
  {
    name: 'Inflammation',
    tests: [
      { name: 'CRP', unit: 'mg/L', ref_low: null, ref_high: 3.0 },
      { name: 'ESR', unit: 'mm/hr', ref_low: null, ref_high: 20 },
      { name: 'Ferritin', unit: 'ng/mL', ref_low: 12, ref_high: 150 },
      { name: 'Fibrinogen', unit: 'mg/dL', ref_low: 200, ref_high: 400 },
      { name: 'IL-6', unit: 'pg/mL', ref_low: null, ref_high: 7 },
      { name: 'Procalcitonin', unit: 'ng/mL', ref_low: null, ref_high: 0.1 },
    ],
  },
  {
    name: 'Lupus/APS Markers',
    tests: [
      { name: 'ANA', unit: '', ref_low: null, ref_high: null, text_only: true },
      { name: 'Anti-dsDNA', unit: 'IU/mL', ref_low: null, ref_high: 30 },
      { name: 'Anti-Smith', unit: 'AI', ref_low: null, ref_high: 1.0 },
      { name: 'Anti-SSA/Ro', unit: 'AI', ref_low: null, ref_high: 1.0 },
      { name: 'Anti-SSB/La', unit: 'AI', ref_low: null, ref_high: 1.0 },
      { name: 'Anti-Scl-70', unit: 'AI', ref_low: null, ref_high: 1.0 },
      { name: 'Anti-Jo-1', unit: 'AI', ref_low: null, ref_high: 1.0 },
      { name: 'C3', unit: 'mg/dL', ref_low: 90, ref_high: 180 },
      { name: 'C4', unit: 'mg/dL', ref_low: 10, ref_high: 40 },
      { name: 'CH50', unit: 'U/mL', ref_low: 30, ref_high: 75 },
      { name: 'Anticardiolipin IgG', unit: 'GPL', ref_low: null, ref_high: 20 },
      { name: 'Anticardiolipin IgM', unit: 'MPL', ref_low: null, ref_high: 20 },
      { name: 'Beta-2 Glycoprotein I IgG', unit: 'SGU', ref_low: null, ref_high: 20 },
      { name: 'Beta-2 Glycoprotein I IgM', unit: 'SMU', ref_low: null, ref_high: 20 },
      { name: 'Lupus Anticoagulant', unit: '', ref_low: null, ref_high: null, text_only: true },
      { name: 'Direct Coombs', unit: '', ref_low: null, ref_high: null, text_only: true },
      { name: 'Urine Protein/Creatinine', unit: 'mg/g', ref_low: null, ref_high: 200 },
    ],
  },
  {
    name: 'Metabolic / BMP',
    tests: [
      { name: 'Sodium', unit: 'mEq/L', ref_low: 136, ref_high: 145 },
      { name: 'Potassium', unit: 'mEq/L', ref_low: 3.5, ref_high: 5.0 },
      { name: 'Chloride', unit: 'mEq/L', ref_low: 98, ref_high: 106 },
      { name: 'CO2', unit: 'mEq/L', ref_low: 23, ref_high: 29 },
      { name: 'BUN', unit: 'mg/dL', ref_low: 7, ref_high: 20 },
      { name: 'Creatinine', unit: 'mg/dL', ref_low: 0.6, ref_high: 1.2 },
      { name: 'eGFR', unit: 'mL/min', ref_low: 60, ref_high: null },
      { name: 'Glucose', unit: 'mg/dL', ref_low: 70, ref_high: 100 },
      { name: 'Calcium', unit: 'mg/dL', ref_low: 8.5, ref_high: 10.5 },
      { name: 'ALT', unit: 'U/L', ref_low: 7, ref_high: 56 },
      { name: 'AST', unit: 'U/L', ref_low: 10, ref_high: 40 },
      { name: 'Alkaline Phosphatase', unit: 'U/L', ref_low: 44, ref_high: 147 },
      { name: 'Total Bilirubin', unit: 'mg/dL', ref_low: 0.1, ref_high: 1.2 },
      { name: 'Albumin', unit: 'g/dL', ref_low: 3.5, ref_high: 5.0 },
    ],
  },
  {
    name: 'Thyroid',
    tests: [
      { name: 'TSH', unit: 'uIU/mL', ref_low: 0.4, ref_high: 4.0 },
      { name: 'Free T4', unit: 'ng/dL', ref_low: 0.8, ref_high: 1.8 },
      { name: 'Free T3', unit: 'pg/mL', ref_low: 2.3, ref_high: 4.2 },
      { name: 'TPO Antibodies', unit: 'IU/mL', ref_low: null, ref_high: 35 },
    ],
  },
  {
    name: 'Coagulation',
    tests: [
      { name: 'PT', unit: 'seconds', ref_low: 11, ref_high: 13.5 },
      { name: 'INR', unit: '', ref_low: 0.8, ref_high: 1.2 },
      { name: 'aPTT', unit: 'seconds', ref_low: 25, ref_high: 35 },
    ],
  },
];

export async function getMergedPanels(): Promise<PanelDefinition[]> {
  const custom = await getCustomLabTests();
  if (custom.length === 0) return LAB_PANELS;

  // Deep clone built-in panels
  const panels: PanelDefinition[] = LAB_PANELS.map(p => ({
    name: p.name,
    tests: [...p.tests],
  }));

  const panelMap = new Map<string, PanelDefinition>();
  for (const p of panels) panelMap.set(p.name, p);

  for (const ct of custom) {
    let panel = panelMap.get(ct.panel);
    if (!panel) {
      panel = { name: ct.panel, tests: [] };
      panels.push(panel);
      panelMap.set(ct.panel, panel);
    }
    // Only add if not already in the panel (avoid duplicates with built-in)
    if (!panel.tests.some(t => t.name === ct.name)) {
      panel.tests.push({
        name: ct.name,
        unit: ct.unit,
        ref_low: ct.ref_low,
        ref_high: ct.ref_high,
        text_only: ct.text_only,
      });
    }
  }

  return panels;
}
