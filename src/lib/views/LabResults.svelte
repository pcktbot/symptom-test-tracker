<script lang="ts">
  import { onMount } from 'svelte';
  import { getLabSessions, getLabSession, deleteLabSession } from '$lib/db';
  import { formatDate, flagClass } from '$lib/utils';
  import type { LabSession, LabResult, View } from '$lib/types';

  let { onNavigate, openGlossary }: { onNavigate: (view: View, sessionId?: number | null) => void; openGlossary: (testName?: string) => void } = $props();

  let sessions: LabSession[] = $state([]);
  let loading = $state(true);
  let expandedId: number | null = $state(null);
  let expandedResults: LabResult[] = $state([]);
  let loadingDetail = $state(false);

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
</script>

<div class="lab-results">
  <div class="header">
    <h1>Lab Results</h1>
    <button class="primary" onclick={() => onNavigate('lab-entry')}>+ New Lab Entry</button>
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
        <div class="session-card" class:expanded={isExpanded}>
          <div class="session-row">
            <button class="show-btn" onclick={() => session.id && toggleShow(session.id)} title={isExpanded ? 'Hide results' : 'Show results'}>
              {isExpanded ? '-' : '+'}
            </button>
            <span class="date">{formatDate(session.test_date)}</span>
            <span class="lab-name">{session.lab_name || '--'}</span>
            <span class="notes">{session.notes || ''}</span>
            <span class="actions">
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

  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 20px;
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
