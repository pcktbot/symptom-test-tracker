export interface LabSession {
  id: number | null;
  test_date: string;
  lab_name: string;
  notes: string;
}

export interface LabResult {
  id: number | null;
  session_id: number | null;
  test_name: string;
  panel: string;
  value: number | null;
  text_value: string;
  unit: string;
  ref_range_low: number | null;
  ref_range_high: number | null;
  flag: string;
}

export interface LabSessionWithResults {
  session: LabSession;
  results: LabResult[];
}

export interface TrendPoint {
  test_date: string;
  value: number | null;
  text_value: string;
  flag: string;
  ref_range_low: number | null;
  ref_range_high: number | null;
}

export interface Symptom {
  id: number | null;
  name: string;
  category: string;
  description: string;
  active: boolean;
  sort_order: number;
}

export interface SymptomEntry {
  symptom_id: number;
  severity: number;
  notes: string;
}

export interface SymptomLogEntry {
  symptom_id: number;
  symptom_name: string;
  category: string;
  severity: number;
  notes: string;
}

export interface DayLog {
  date: string;
  entries: SymptomLogEntry[];
  wellness_score: number;
  notes: string;
}

export type View = 'dashboard' | 'lab-results' | 'lab-entry' | 'trends' | 'symptoms' | 'symptom-editor' | 'export';

export type Flag = 'N' | 'L' | 'H' | 'LL' | 'HH';

export interface PanelDefinition {
  name: string;
  tests: TestDefinition[];
}

export interface TestDefinition {
  name: string;
  unit: string;
  ref_low: number | null;
  ref_high: number | null;
  text_only?: boolean;
}
