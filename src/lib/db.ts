import { invoke } from '@tauri-apps/api/core';
import type {
  LabSession,
  LabResult,
  LabSessionWithResults,
  TrendPoint,
  AbnormalResult,
  CustomLabTest,
  WellnessTrendPoint,
  SymptomTrendPoint,
  SymptomNameEntry,
  Symptom,
  SymptomEntry,
  DayLog,
  Artifact,
  Diagnosis,
  ExtractionResult,
  ChatMessage,
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

export async function mergeLabSessions(targetId: number, sourceId: number): Promise<void> {
  return invoke('merge_lab_sessions', { targetId, sourceId });
}

export async function getLatestAbnormal(): Promise<LabResult[]> {
  return invoke('get_latest_abnormal');
}

export async function getLatestAbnormalWithPrevious(): Promise<AbnormalResult[]> {
  return invoke('get_latest_abnormal_with_previous');
}

// Custom lab tests
export async function getCustomLabTests(): Promise<CustomLabTest[]> {
  return invoke('get_custom_lab_tests');
}

export async function saveCustomLabTest(test: CustomLabTest): Promise<number> {
  return invoke('save_custom_lab_test', { test });
}

export async function deleteCustomLabTest(id: number): Promise<void> {
  return invoke('delete_custom_lab_test', { id });
}

export async function getTrends(testName: string, days: number): Promise<TrendPoint[]> {
  return invoke('get_trends', { testName, days });
}

export async function getAllTestNames(): Promise<string[]> {
  return invoke('get_all_test_names');
}

// Symptom trend commands
export async function getWellnessTrends(days: number): Promise<WellnessTrendPoint[]> {
  return invoke('get_wellness_trends', { days });
}

export async function getSymptomTrends(symptomId: number, days: number): Promise<SymptomTrendPoint[]> {
  return invoke('get_symptom_trends', { symptomId, days });
}

export async function getActiveSymptomNames(): Promise<SymptomNameEntry[]> {
  return invoke('get_active_symptom_names');
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

// Settings
export async function getSetting(key: string): Promise<string> {
  return invoke('get_setting', { key });
}

export async function setSetting(key: string, value: string): Promise<void> {
  return invoke('set_setting', { key, value });
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

// Diagnoses
export async function getDiagnoses(): Promise<Diagnosis[]> {
  return invoke('get_diagnoses');
}

export async function saveDiagnosis(diagnosis: Diagnosis): Promise<number> {
  return invoke('save_diagnosis', { diagnosis });
}

export async function deleteDiagnosis(id: number): Promise<void> {
  return invoke('delete_diagnosis', { id });
}

// Artifacts
export async function getArtifacts(): Promise<Artifact[]> {
  return invoke('get_artifacts');
}

export async function getArtifact(id: number): Promise<Artifact> {
  return invoke('get_artifact', { id });
}

export async function saveArtifactPaste(title: string, content: string): Promise<number> {
  return invoke('save_artifact_paste', { title, content });
}

export async function saveArtifactFile(title: string, sourcePath: string): Promise<number> {
  return invoke('save_artifact_file', { title, sourcePath });
}

export async function deleteArtifact(id: number): Promise<void> {
  return invoke('delete_artifact', { id });
}

export async function extractLabsFromArtifact(artifactId: number): Promise<ExtractionResult> {
  return invoke('extract_labs_from_artifact', { artifactId });
}

// Chat
export async function sendChatMessage(content: string): Promise<void> {
  return invoke('send_chat_message', { content });
}

export async function getChatHistory(): Promise<ChatMessage[]> {
  return invoke('get_chat_history');
}

export async function clearChatHistory(): Promise<void> {
  return invoke('clear_chat_history');
}

export async function getMemory(): Promise<string> {
  return invoke('get_memory');
}

export async function saveMemory(content: string): Promise<void> {
  return invoke('save_memory', { content });
}
