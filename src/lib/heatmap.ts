import type { HeatmapConfig } from './types';

export const DEFAULT_CONFIG: HeatmapConfig = {
  steps: 5,
  labels: ['Worst', 'Poor', 'Okay', 'Good', 'Best'],
  colors: ['#c0392b', '#e67e22', '#f1c40f', '#27ae60', '#1e8449'],
};

export function scoreToColor(score: number, config: HeatmapConfig): string {
  if (score < 1 || score > config.steps) return '';
  return config.colors[score - 1];
}

export function scoreToLabel(score: number, config: HeatmapConfig): string {
  if (score < 1 || score > config.steps) return '';
  return config.labels[score - 1];
}

export function remapScore(score: number, oldSteps: number, newSteps: number): number {
  if (oldSteps === 1) return 1;
  const normalized = (score - 1) / (oldSteps - 1);
  return Math.min(newSteps, Math.max(1, Math.round(normalized * (newSteps - 1) + 1)));
}

export function parseTags(raw: string | string[]): string[] {
  if (Array.isArray(raw)) return raw;
  if (!raw) return [];
  try {
    return JSON.parse(raw);
  } catch {
    return [];
  }
}

export function serializeTags(tags: string[]): string {
  return JSON.stringify(tags);
}

export function isValidDate(year: number, month: number, day: number): boolean {
  const d = new Date(year, month - 1, day);
  return (
    d.getFullYear() === year &&
    d.getMonth() === month - 1 &&
    d.getDate() === day
  );
}
