<script lang="ts">
  import { onMount } from 'svelte';
  import { getSymptoms, getSymptomLog, saveSymptomLog } from '$lib/db';
  import { todayString, formatDate } from '$lib/utils';
  import type { Symptom, SymptomEntry as SymptomEntryType } from '$lib/types';

  let symptoms: Symptom[] = $state([]);
  let date = $state(todayString());
  let wellnessScore = $state(5);
  let dailyNotes = $state('');
  let saving = $state(false);
  let saved = $state(false);

  // Keyed by symptom id
  let entries: Record<number, { severity: number; notes: string }> = $state({});

  onMount(() => loadDay());

  async function loadDay() {
    try {
      symptoms = (await getSymptoms()).filter(s => s.active);
      const log = await getSymptomLog(date);
      wellnessScore = log.wellness_score;
      dailyNotes = log.notes;

      // Reset entries
      const newEntries: Record<number, { severity: number; notes: string }> = {};
      for (const s of symptoms) {
        newEntries[s.id!] = { severity: 0, notes: '' };
      }
      for (const e of log.entries) {
        if (newEntries[e.symptom_id]) {
          newEntries[e.symptom_id] = { severity: e.severity, notes: e.notes };
        }
      }
      entries = newEntries;
      saved = false;
    } catch (e) {
      console.error('Failed to load symptoms:', e);
    }
  }

  function handleDateChange() {
    loadDay();
  }

  async function handleSave() {
    saving = true;
    try {
      const entryList: SymptomEntryType[] = Object.entries(entries).map(([id, e]) => ({
        symptom_id: parseInt(id),
        severity: e.severity,
        notes: e.notes,
      }));
      await saveSymptomLog(date, entryList, wellnessScore, dailyNotes);
      saved = true;
      setTimeout(() => saved = false, 2000);
    } catch (e) {
      console.error('Failed to save:', e);
      alert('Failed to save symptom log');
    }
    saving = false;
  }

  function prevDay() {
    const d = new Date(date + 'T00:00:00');
    d.setDate(d.getDate() - 1);
    date = d.toISOString().slice(0, 10);
    loadDay();
  }

  function nextDay() {
    const d = new Date(date + 'T00:00:00');
    d.setDate(d.getDate() + 1);
    date = d.toISOString().slice(0, 10);
    loadDay();
  }

  // Group symptoms by category
  let grouped = $derived(() => {
    const groups: Record<string, { symptom: Symptom; entry: { severity: number; notes: string } }[]> = {};
    for (const s of symptoms) {
      const cat = s.category || 'Other';
      if (!groups[cat]) groups[cat] = [];
      const entry = entries[s.id!];
      if (entry) {
        groups[cat].push({ symptom: s, entry });
      }
    }
    return groups;
  });
</script>

<div class="symptom-entry">
  <div class="header">
    <h1>Symptom Log</h1>
    <div class="header-actions">
      {#if saved}
        <span class="saved-indicator">Saved</span>
      {/if}
      <button class="primary" onclick={handleSave} disabled={saving}>
        {saving ? 'Saving...' : 'Save'}
      </button>
    </div>
  </div>

  <div class="date-nav">
    <button onclick={prevDay}>&larr;</button>
    <input type="date" bind:value={date} onchange={handleDateChange} />
    <button onclick={nextDay}>&rarr;</button>
    <span class="date-display">{formatDate(date)}</span>
  </div>

  <div class="wellness-section">
    <label for="wellness">Overall Wellness: {wellnessScore}/10</label>
    <input id="wellness" type="range" min="1" max="10" bind:value={wellnessScore} />
    <div class="range-labels">
      <span>Terrible</span>
      <span>Great</span>
    </div>
  </div>

  <div class="symptoms-grid">
    {#each Object.entries(grouped()) as [category, items]}
      <div class="category-group">
        <h3>{category}</h3>
        {#each items as { symptom, entry }}
          <div class="symptom-row">
            <div class="symptom-info">
              <span class="symptom-name">{symptom.name}</span>
              <span class="severity-value">{entry.severity > 0 ? entry.severity + '/10' : '--'}</span>
            </div>
            <input
              type="range"
              min="0"
              max="10"
              bind:value={entry.severity}
              class="severity-slider"
              class:active={entry.severity > 0}
            />
            <input
              type="text"
              class="symptom-notes"
              bind:value={entry.notes}
              placeholder="Notes..."
            />
          </div>
        {/each}
      </div>
    {/each}
  </div>

  <div class="daily-notes">
    <label for="daily-notes">Daily Notes</label>
    <textarea id="daily-notes" bind:value={dailyNotes} rows="3" placeholder="How are you feeling today? Any observations..."></textarea>
  </div>
</div>

<style>
  .symptom-entry { max-width: 700px; }

  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 16px;
  }

  .header-actions {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .saved-indicator {
    color: var(--color-success);
    font-size: 13px;
    font-weight: 500;
  }

  .date-nav {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 20px;
  }

  .date-nav button {
    padding: 4px 10px;
    font-size: 16px;
  }

  .date-nav input[type="date"] {
    font-family: var(--font-mono);
  }

  .date-display {
    color: var(--color-text-muted);
    font-size: 13px;
  }

  .wellness-section {
    margin-bottom: 24px;
    padding: 12px;
    border: 1px solid var(--color-border);
    border-radius: var(--radius);
  }

  .wellness-section label {
    font-size: 14px;
    font-weight: 500;
    color: var(--color-text);
    margin-bottom: 6px;
  }

  .wellness-section input[type="range"] {
    width: 100%;
    border: none;
    padding: 0;
  }

  .range-labels {
    display: flex;
    justify-content: space-between;
    font-size: 11px;
    color: var(--color-text-muted);
    margin-top: 2px;
  }

  .category-group {
    margin-bottom: 16px;
  }

  .category-group h3 {
    margin-bottom: 8px;
    padding-bottom: 4px;
    border-bottom: 1px solid var(--color-border);
    font-size: 13px;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--color-text-muted);
  }

  .symptom-row {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 4px 0;
  }

  .symptom-info {
    width: 160px;
    display: flex;
    justify-content: space-between;
    align-items: center;
    flex-shrink: 0;
  }

  .symptom-name {
    font-size: 13px;
    font-weight: 500;
  }

  .severity-value {
    font-family: var(--font-mono);
    font-size: 12px;
    color: var(--color-text-muted);
    width: 35px;
    text-align: right;
  }

  .severity-slider {
    flex: 1;
    min-width: 100px;
    border: none;
    padding: 0;
  }

  .symptom-notes {
    width: 160px;
    font-size: 12px;
    padding: 3px 6px;
    flex-shrink: 0;
  }

  .daily-notes {
    margin-top: 20px;
  }

  .daily-notes label {
    font-size: 14px;
    font-weight: 500;
    color: var(--color-text);
    margin-bottom: 4px;
  }

  .daily-notes textarea {
    width: 100%;
  }
</style>
