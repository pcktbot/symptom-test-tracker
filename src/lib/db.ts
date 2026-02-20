import { invoke } from '@tauri-apps/api/core';
import type {
  LabSession,
  LabResult,
  LabSessionWithResults,
  TrendPoint,
  Symptom,
  SymptomEntry,
  DayLog,
} from './types';

// Lab commands
export async function getLabSessions(): Promise<LabSession[]> {
  return invoke('get_lab_sessions');
}

export async function getLabSession(id: number): Promise<LabSessionWithResults> {
  return invoke('get_lab_session', { id });
}

export async function saveLabSession(session: LabSession, results: LabResult[]): Promise<number> {
  return invoke('save_lab_session', { session, results });
}

export async function deleteLabSession(id: number): Promise<void> {
  return invoke('delete_lab_session', { id });
}

export async function getLatestAbnormal(): Promise<LabResult[]> {
  return invoke('get_latest_abnormal');
}

export async function getTrends(testName: string, days: number): Promise<TrendPoint[]> {
  return invoke('get_trends', { testName, days });
}

export async function getAllTestNames(): Promise<string[]> {
  return invoke('get_all_test_names');
}

// Symptom commands
export async function getSymptoms(): Promise<Symptom[]> {
  return invoke('get_symptoms');
}

export async function saveSymptom(symptom: Symptom): Promise<number> {
  return invoke('save_symptom', { symptom });
}

export async function deleteSymptom(id: number): Promise<void> {
  return invoke('delete_symptom', { id });
}

export async function reorderSymptoms(ids: number[]): Promise<void> {
  return invoke('reorder_symptoms', { ids });
}

export async function getSymptomLog(date: string): Promise<DayLog> {
  return invoke('get_symptom_log', { date });
}

export async function saveSymptomLog(
  date: string,
  entries: SymptomEntry[],
  wellness: number,
  notes: string,
): Promise<void> {
  return invoke('save_symptom_log', { date, entries, wellness, notes });
}

// Export
export async function exportData(
  startDate: string,
  endDate: string,
  includeLabs: boolean,
  includeSymptoms: boolean,
  format: string,
): Promise<string> {
  return invoke('export_data', { startDate, endDate, includeLabs, includeSymptoms, format });
}
