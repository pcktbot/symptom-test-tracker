<script lang="ts">
  import { onMount } from 'svelte';
  import { getSymptoms, getSymptomLog, saveSymptomLog } from '$lib/db';
  import { todayString, formatDate } from '$lib/utils';
  import type { Symptom, SymptomEntry as SymptomEntryType, View } from '$lib/types';
  import DiagnosisAccordion from '$lib/components/DiagnosisAccordion.svelte';

  let { onNavigate, openSymptomConfig }: {
    onNavigate: (view: View) => void;
    openSymptomConfig: () => void;
  } = $props();

  let symptoms: Symptom[] = $state([]);
  let date = $state(todayString());
  let wellnessScore = $state(3);
  let dailyNotes = $state('');
  let saving = $state(false);
  let saved = $state(false);

  // Keyed by symptom id — severity is 1 (present) or 0 (absent)
  let entries: Record<number, boolean> = $state({});

  const WELLNESS_LABELS: Record<number, string> = {
    1: 'Terrible',
    2: 'Poor',
    3: 'Okay',
    4: 'Good',
    5: 'Great',
  };

  onMount(() => loadDay());

  async function loadDay() {
    try {
      symptoms = (await getSymptoms()).filter(s => s.active);
      const log = await getSymptomLog(date);
      // Clamp old 1-10 scores to 1-5 range
      wellnessScore = Math.max(1, Math.min(5, log.wellness_score));
      dailyNotes = log.notes;

      const newEntries: Record<number, boolean> = {};
      for (const s of symptoms) {
        newEntries[s.id!] = false;
      }
      for (const e of log.entries) {
        if (newEntries[e.symptom_id] !== undefined && e.severity > 0) {
          newEntries[e.symptom_id] = true;
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
      const entryList: SymptomEntryType[] = Object.entries(entries).map(([id, present]) => ({
        symptom_id: parseInt(id),
        severity: present ? 1 : 0,
        notes: '',
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
    const groups: Record<string, Symptom[]> = {};
    for (const s of symptoms) {
      const cat = s.category || 'Other';
      if (!groups[cat]) groups[cat] = [];
      groups[cat].push(s);
    }
    return groups;
  });
</script>

<div class="symptom-entry">
  <DiagnosisAccordion {onNavigate} />
  <div class="view-header">
    <h2>Symptom Log</h2>
    <button class="configure-btn" onclick={openSymptomConfig}>
      <svg width="14" height="14" viewBox="0 0 16 16" fill="currentColor">
        <path d="M8 4.754a3.246 3.246 0 1 0 0 6.492 3.246 3.246 0 0 0 0-6.492zM5.754 8a2.246 2.246 0 1 1 4.492 0 2.246 2.246 0 0 1-4.492 0z"/>
        <path d="M9.796 1.343c-.527-1.79-3.065-1.79-3.592 0l-.094.319a.873.873 0 0 1-1.255.52l-.292-.16c-1.64-.892-3.433.902-2.54 2.541l.159.292a.873.873 0 0 1-.52 1.255l-.319.094c-1.79.527-1.79 3.065 0 3.592l.319.094a.873.873 0 0 1 .52 1.255l-.16.292c-.892 1.64.902 3.434 2.541 2.54l.292-.159a.873.873 0 0 1 1.255.52l.094.319c.527 1.79 3.065 1.79 3.592 0l.094-.319a.873.873 0 0 1 1.255-.52l.292.16c1.64.893 3.434-.902 2.54-2.541l-.159-.292a.873.873 0 0 1 .52-1.255l.319-.094c1.79-.527 1.79-3.065 0-3.592l-.319-.094a.873.873 0 0 1-.52-1.255l.16-.292c.893-1.64-.902-3.433-2.541-2.54l-.292.159a.873.873 0 0 1-1.255-.52l-.094-.319zm-2.633.283c.246-.835 1.428-.835 1.674 0l.094.319a1.873 1.873 0 0 0 2.693 1.115l.291-.16c.764-.415 1.6.42 1.184 1.185l-.159.292a1.873 1.873 0 0 0 1.116 2.692l.318.094c.835.246.835 1.428 0 1.674l-.319.094a1.873 1.873 0 0 0-1.115 2.693l.16.291c.415.764-.421 1.6-1.185 1.184l-.291-.159a1.873 1.873 0 0 0-2.693 1.116l-.094.318c-.246.835-1.428.835-1.674 0l-.094-.319a1.873 1.873 0 0 0-2.692-1.115l-.292.16c-.764.415-1.6-.421-1.184-1.185l.159-.291A1.873 1.873 0 0 0 1.945 8.93l-.319-.094c-.835-.246-.835-1.428 0-1.674l.319-.094A1.873 1.873 0 0 0 3.06 4.377l-.16-.292c-.415-.764.421-1.6 1.185-1.184l.292.159a1.873 1.873 0 0 0 2.692-1.116l.094-.318z"/>
      </svg>
      Configure
    </button>
  </div>
  <div class="header">
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
    <div class="wellness-label">How are you feeling today?</div>
    <div class="wellness-buttons">
      {#each [1, 2, 3, 4, 5] as score}
        <button
          class="wellness-btn"
          class:selected={wellnessScore === score}
          onclick={() => wellnessScore = score}
        >
          <span class="wellness-score">{score}</span>
          <span class="wellness-text">{WELLNESS_LABELS[score]}</span>
        </button>
      {/each}
    </div>
  </div>

  <div class="symptoms-grid">
    {#each Object.entries(grouped()) as [category, items]}
      <div class="category-group">
        <h3>{category}</h3>
        <div class="checklist">
          {#each items as symptom}
            <label class="symptom-check">
              <input
                type="checkbox"
                bind:checked={entries[symptom.id!]}
              />
              <span class="check-label">{symptom.name}</span>
            </label>
          {/each}
        </div>
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

  .view-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 20px;
  }

  .view-header h2 {
    margin: 0;
  }

  .configure-btn {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 6px 12px;
    font-size: 13px;
    border: 1px solid var(--color-border);
    border-radius: var(--radius);
    background: var(--color-surface);
    color: var(--color-text-muted);
    cursor: pointer;
    transition: background 0.15s, color 0.15s, border-color 0.15s;
  }

  .configure-btn:hover {
    background: var(--color-surface-raised);
    color: var(--color-text);
    border-color: var(--color-border-strong);
  }

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
    padding: 16px;
    border: 1px solid var(--color-border);
    border-radius: var(--radius);
  }

  .wellness-label {
    font-size: 14px;
    font-weight: 500;
    color: var(--color-text);
    margin-bottom: 10px;
  }

  .wellness-buttons {
    display: flex;
    gap: 6px;
  }

  .wellness-btn {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 2px;
    padding: 10px 4px;
    border: 1px solid var(--color-border);
    background: var(--color-surface);
    border-radius: var(--radius);
    cursor: pointer;
    transition: all 0.15s;
  }

  .wellness-btn:hover {
    border-color: var(--color-accent);
    background: var(--color-surface-raised);
  }

  .wellness-btn.selected {
    border-color: var(--color-accent);
    background: var(--color-accent);
    color: white;
  }

  .wellness-score {
    font-size: 18px;
    font-weight: 600;
    line-height: 1;
  }

  .wellness-text {
    font-size: 11px;
    line-height: 1;
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

  .checklist {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
    gap: 4px 16px;
  }

  .symptom-check {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 5px 4px;
    cursor: pointer;
    border-radius: 4px;
    font-size: 13px;
    color: var(--color-text);
    margin-bottom: 0;
  }

  .symptom-check:hover {
    background: var(--color-surface-raised);
  }

  .symptom-check input[type="checkbox"] {
    margin: 0;
    width: 16px;
    height: 16px;
    accent-color: var(--color-accent);
    flex-shrink: 0;
  }

  .check-label {
    font-weight: 500;
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
