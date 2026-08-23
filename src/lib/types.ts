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

export interface WellnessTrendPoint {
  date: string;
  wellness_score: number;
}

export interface SymptomTrendPoint {
  date: string;
  present: boolean;
}

export interface SymptomNameEntry {
  id: number;
  name: string;
}

export interface AbnormalResult extends LabResult {
  test_date: string;
  prev_value: number | null;
  prev_text_value: string;
  prev_flag: string;
}

export type View =
  | 'dashboard'
  | 'lab-results'
  | 'lab-entry'
  | 'trends'
  | 'daily-rating'
  | 'documents'
  | 'care-team'
  | 'welcome';

export type Flag = 'N' | 'L' | 'H' | 'LL' | 'HH';

export interface CustomLabTest {
  id: number | null;
  name: string;
  panel: string;
  unit: string;
  ref_low: number | null;
  ref_high: number | null;
  text_only: boolean;
  description: string;
  clinical: string;
}

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

export interface Artifact {
  id: number | null;
  title: string;
  source_type: 'paste' | 'file';
  content_type: 'text' | 'html' | 'pdf';
  text_content: string;
  file_path: string;
  created_at: string;
}

export interface Diagnosis {
  id: number | null;
  name: string;
  short_name: string;
  onset_date: string | null;
  resolution_date: string | null;
  chronic: boolean;
  source: string;
  details: string;
  created_at: string;
}

export interface ExtractedLabResult {
  test_name: string;
  panel: string;
  value: number | null;
  text_value: string;
  unit: string;
  ref_range_low: number | null;
  ref_range_high: number | null;
  flag: string;
}

export interface ExtractionResult {
  date: string | null;
  results: ExtractedLabResult[];
}

export interface ChatMessage {
  role: 'user' | 'assistant';
  content: string;
  created_at: string;
}

export interface DailyRating {
  log_date: string;
  wellness_score: number; // 1-based: 1 = worst, N = best
  tags: string[];
}

export interface HeatmapConfig {
  steps: number;
  labels: string[];
  colors: string[];
}
