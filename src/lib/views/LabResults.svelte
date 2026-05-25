<script lang="ts">
  import { onMount } from 'svelte';
  import { getLabSessions, getLabSession, deleteLabSession, mergeLabSessions } from '$lib/db';
  import { formatDate, flagClass } from '$lib/utils';
  import type { LabSession, LabResult, View } from '$lib/types';
  import DiagnosisAccordion from '$lib/components/DiagnosisAccordion.svelte';

  let { onNavigate, openGlossary, openLabConfig }: {
    onNavigate: (view: View, sessionId?: number | null) => void;
    openGlossary: (testName?: string) => void;
    openLabConfig: () => void;
  } = $props();

  let sessions: LabSession[] = $state([]);
  let loading = $state(true);
  let expandedId: number | null = $state(null);
  let expandedResults: LabResult[] = $state([]);
  let loadingDetail = $state(false);
  let duplicateDateMap = $derived(duplicateDates(sessions));

  onMount(loadSessions);

  async function loadSessions() {
    loading = true;
    try {
      sessions = await getLabSessions();
    } catch (e) {
      console.error('Failed to load lab sessions:', e);
    }
    loading = false;
  }

  async function toggleShow(id: number) {
    if (expandedId === id) {
      expandedId = null;
      expandedResults = [];
      return;
    }
    loadingDetail = true;
    expandedId = id;
    try {
      const data = await getLabSession(id);
      expandedResults = data.results;
    } catch (e) {
      console.error('Failed to load session detail:', e);
      expandedResults = [];
    }
    loadingDetail = false;
  }

  async function handleDelete(id: number) {
    if (!confirm('Delete this lab session and all its results?')) return;
    try {
      await deleteLabSession(id);
      if (expandedId === id) {
        expandedId = null;
        expandedResults = [];
      }
      await loadSessions();
    } catch (e) {
      console.error('Failed to delete session:', e);
    }
  }

  function groupByPanel(results: LabResult[]): Record<string, LabResult[]> {
    const groups: Record<string, LabResult[]> = {};
    for (const r of results) {
      const panel = r.panel || 'Other';
      if (!groups[panel]) groups[panel] = [];
      groups[panel].push(r);
    }
    return groups;
  }

  // Returns a map of test_date → session ids, for dates that have >1 session
  function duplicateDates(sessions: LabSession[]): Map<string, number[]> {
    const byDate = new Map<string, number[]>();
    for (const s of sessions) {
      if (!s.id) continue;
      const ids = byDate.get(s.test_date) ?? [];
      ids.push(s.id);
      byDate.set(s.test_date, ids);
    }
    const dupes = new Map<string, number[]>();
    for (const [date, ids] of byDate) {
      if (ids.length > 1) dupes.set(date, ids);
    }
    return dupes;
  }

  async function handleMerge(targetId: number, sourceId: number) {
    if (!confirm(`Merge this session into session #${targetId}? The merged session will be deleted.`)) return;
    try {
      await mergeLabSessions(targetId, sourceId);
      if (expandedId === sourceId) { expandedId = null; expandedResults = []; }
      await loadSessions();
    } catch (e) {
      console.error('Failed to merge sessions:', e);
      alert('Failed to merge sessions');
    }
  }
</script>

<div class="lab-results">
  <DiagnosisAccordion {onNavigate} />
  <div class="view-header">
    <h2>Lab Results</h2>
    <button class="configure-btn" onclick={openLabConfig}>
      <svg width="14" height="14" viewBox="0 0 16 16" fill="currentColor">
        <path d="M8 4.754a3.246 3.246 0 1 0 0 6.492 3.246 3.246 0 0 0 0-6.492zM5.754 8a2.246 2.246 0 1 1 4.492 0 2.246 2.246 0 0 1-4.492 0z"/>
        <path d="M9.796 1.343c-.527-1.79-3.065-1.79-3.592 0l-.094.319a.873.873 0 0 1-1.255.52l-.292-.16c-1.64-.892-3.433.902-2.54 2.541l.159.292a.873.873 0 0 1-.52 1.255l-.319.094c-1.79.527-1.79 3.065 0 3.592l.319.094a.873.873 0 0 1 .52 1.255l-.16.292c-.892 1.64.902 3.434 2.541 2.54l.292-.159a.873.873 0 0 1 1.255.52l.094.319c.527 1.79 3.065 1.79 3.592 0l.094-.319a.873.873 0 0 1 1.255-.52l.292.16c1.64.893 3.434-.902 2.54-2.541l-.159-.292a.873.873 0 0 1 .52-1.255l.319-.094c1.79-.527 1.79-3.065 0-3.592l-.319-.094a.873.873 0 0 1-.52-1.255l.16-.292c.893-1.64-.902-3.433-2.541-2.54l-.292.159a.873.873 0 0 1-1.255-.52l-.094-.319zm-2.633.283c.246-.835 1.428-.835 1.674 0l.094.319a1.873 1.873 0 0 0 2.693 1.115l.291-.16c.764-.415 1.6.42 1.184 1.185l-.159.292a1.873 1.873 0 0 0 1.116 2.692l.318.094c.835.246.835 1.428 0 1.674l-.319.094a1.873 1.873 0 0 0-1.115 2.693l.16.291c.415.764-.421 1.6-1.185 1.184l-.291-.159a1.873 1.873 0 0 0-2.693 1.116l-.094.318c-.246.835-1.428.835-1.674 0l-.094-.319a1.873 1.873 0 0 0-2.692-1.115l-.292.16c-.764.415-1.6-.421-1.184-1.185l.159-.291A1.873 1.873 0 0 0 1.945 8.93l-.319-.094c-.835-.246-.835-1.428 0-1.674l.319-.094A1.873 1.873 0 0 0 3.06 4.377l-.16-.292c-.415-.764.421-1.6 1.185-1.184l.292.159a1.873 1.873 0 0 0 2.692-1.116l.094-.318z"/>
      </svg>
      Configure
    </button>
  </div>
  <div class="header">
    <div class="header-actions">
      <button onclick={loadSessions} disabled={loading}>Refresh</button>
      <button class="primary" onclick={() => onNavigate('lab-entry')}>+ New Lab Entry</button>
    </div>
  </div>

  {#if loading}
    <p class="muted">Loading...</p>
  {:else if sessions.length === 0}
    <div class="empty-state">
      <p>No lab sessions recorded yet.</p>
      <p class="muted">Click "New Lab Entry" to add your first set of results.</p>
    </div>
  {:else}
    <div class="sessions-list">
      {#each sessions as session}
        {@const isExpanded = expandedId === session.id}
        {@const siblingIds = session.test_date && session.id ? (duplicateDateMap.get(session.test_date) ?? []).filter(id => id !== session.id) : []}
        <div class="session-card" class:expanded={isExpanded} class:duplicate={siblingIds.length > 0}>
          <div class="session-row">
            <button class="show-btn" onclick={() => session.id && toggleShow(session.id)} title={isExpanded ? 'Hide results' : 'Show results'}>
              {isExpanded ? '-' : '+'}
            </button>
            <span class="date">{formatDate(session.test_date)}</span>
            <span class="lab-name">{session.lab_name || '--'}</span>
            <span class="notes">{session.notes || ''}</span>
            <span class="actions">
              {#if siblingIds.length > 0}
                <button class="merge-btn" title="Merge into earliest session for this date"
                  onclick={() => session.id && handleMerge(siblingIds[0], session.id)}>
                  ⇒ Merge
                </button>
              {/if}
              <button onclick={() => onNavigate('lab-entry', session.id)}>Edit</button>
              <button class="danger" onclick={() => session.id && handleDelete(session.id)}>Delete</button>
            </span>
          </div>

          {#if isExpanded}
            <div class="session-detail">
              {#if loadingDetail}
                <p class="muted">Loading...</p>
              {:else if expandedResults.length === 0}
                <p class="muted">No results recorded for this session.</p>
              {:else}
                {#each Object.entries(groupByPanel(expandedResults)) as [panel, results]}
                  <div class="panel-group">
                    <h3>{panel}</h3>
                    <table>
                      <thead>
                        <tr>
                          <th>Test</th>
                          <th>Value</th>
                          <th>Unit</th>
                          <th>Reference Range</th>
                          <th>Flag</th>
                        </tr>
                      </thead>
                      <tbody>
                        {#each results as r}
                          <tr>
                            <td class="test-name">
                              {r.test_name}
                              <button class="info-btn" onclick={() => openGlossary(r.test_name)} title="View in glossary">i</button>
                            </td>
                            <td class="value {flagClass(r.flag)}">
                              {#if r.value != null}
                                {r.value}
                              {:else}
                                {r.text_value}
                              {/if}
                            </td>
                            <td class="unit">{r.unit}</td>
                            <td class="ref-range">
                              {#if r.ref_range_low != null && r.ref_range_high != null}
                                {r.ref_range_low} - {r.ref_range_high}
                              {:else if r.ref_range_high != null}
                                &lt; {r.ref_range_high}
                              {:else if r.ref_range_low != null}
                                &gt; {r.ref_range_low}
                              {:else}
                                --
                              {/if}
                            </td>
                            <td><span class="badge {flagClass(r.flag)}">{r.flag}</span></td>
                          </tr>
                        {/each}
                      </tbody>
                    </table>
                  </div>
                {/each}
              {/if}
            </div>
          {/if}
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .lab-results { max-width: 900px; }

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
    margin-bottom: 20px;
  }

  .header-actions {
    display: flex;
    gap: 8px;
  }

  .muted { color: var(--color-text-muted); }

  .empty-state {
    padding: 40px;
    text-align: center;
    border: 1px dashed var(--color-border);
    border-radius: var(--radius);
    margin-top: 20px;
  }

  .sessions-list {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .session-card {
    border: 1px solid var(--color-border);
    border-radius: var(--radius);
    overflow: hidden;
  }

  .session-card.expanded {
    border-color: var(--color-border-strong);
  }

  .session-card.duplicate {
    border-color: var(--color-warning, #f59e0b);
  }

  .merge-btn {
    font-size: 12px;
    padding: 3px 8px;
    color: var(--color-warning, #f59e0b);
    border-color: var(--color-warning, #f59e0b);
  }

  .session-row {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 8px 12px;
  }

  .show-btn {
    width: 26px;
    height: 26px;
    display: flex;
    align-items: center;
    justify-content: center;
    border: 1px solid var(--color-border);
    background: var(--color-surface-raised);
    border-radius: var(--radius);
    font-family: var(--font-mono);
    font-size: 14px;
    color: var(--color-text-muted);
    cursor: pointer;
    padding: 0;
    flex-shrink: 0;
  }

  .show-btn:hover {
    border-color: var(--color-border-strong);
    color: var(--color-text);
  }

  .date {
    font-weight: 500;
    white-space: nowrap;
    flex-shrink: 0;
  }

  .lab-name {
    color: var(--color-text-muted);
    flex-shrink: 0;
  }

  .notes {
    color: var(--color-text-muted);
    font-size: 13px;
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    min-width: 0;
  }

  .actions {
    display: flex;
    gap: 4px;
    flex-shrink: 0;
  }

  .actions button {
    font-size: 12px;
    padding: 3px 8px;
  }

  .session-detail {
    border-top: 1px solid var(--color-border);
    padding: 12px 16px 16px;
    background: var(--color-surface-raised);
  }

  .panel-group {
    margin-bottom: 16px;
  }

  .panel-group:last-child {
    margin-bottom: 0;
  }

  .panel-group h3 {
    font-size: 13px;
    margin-bottom: 6px;
    padding-bottom: 4px;
    border-bottom: 1px solid var(--color-border);
  }

  table {
    width: 100%;
    border-collapse: collapse;
    font-size: 13px;
  }

  th {
    text-align: left;
    padding: 4px 10px;
    color: var(--color-text-muted);
    font-weight: 500;
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.03em;
    border-bottom: 1px solid var(--color-border);
  }

  td {
    padding: 4px 10px;
    border-bottom: 1px solid var(--color-border);
  }

  .test-name {
    font-weight: 500;
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .value { font-family: var(--font-mono); }
  .unit { color: var(--color-text-muted); }
  .ref-range { color: var(--color-text-muted); font-family: var(--font-mono); font-size: 12px; }
</style>
