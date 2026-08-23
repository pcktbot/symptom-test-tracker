<script lang="ts">
  import { onMount } from 'svelte';
  import { getLabSessions, getLabSession, deleteLabSession, mergeLabSessions } from '$lib/db';
  import { formatDate } from '$lib/utils';
  import type { LabSession, LabResult, LabSessionWithResults, View } from '$lib/types';
  import DiagnosisAccordion from '$lib/components/DiagnosisAccordion.svelte';

  let { onNavigate, openGlossary, openLabConfig }: {
    onNavigate: (view: View, sessionId?: number | null) => void;
    openGlossary: (testName?: string) => void;
    openLabConfig: () => void;
  } = $props();

  let sessions: LabSession[] = $state([]);
  let loading = $state(true);
  let expanded: number | null = $state(null);
  let resultsBySession: Record<number, LabResult[]> = $state({});
  let loadingDetail = $state(false);
  let duplicateDateMap = $derived(duplicateDates(sessions));

  onMount(async () => {
    await loadSessions();
    // Auto-expand the first session
    if (sessions.length > 0 && sessions[0].id !== null) {
      await toggle(sessions[0].id);
    }
  });

  async function loadSessions() {
    loading = true;
    try {
      sessions = await getLabSessions();
    } catch (e) {
      console.error('Failed to load lab sessions:', e);
    }
    loading = false;
  }

  async function toggle(id: number | null) {
    if (id === null) return;
    if (expanded === id) {
      expanded = null;
      return;
    }
    if (!resultsBySession[id]) {
      loadingDetail = true;
      try {
        const data = await getLabSession(id);
        resultsBySession[id] = data.results;
      } catch (e) {
        console.error('Failed to load session detail:', e);
        resultsBySession[id] = [];
      }
      loadingDetail = false;
    }
    expanded = id;
  }

  async function handleDelete(id: number) {
    if (!confirm('Delete this lab session and all its results?')) return;
    try {
      await deleteLabSession(id);
      if (expanded === id) expanded = null;
      delete resultsBySession[id];
      await loadSessions();
    } catch (e) {
      console.error('Failed to delete session:', e);
    }
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
      if (expanded === sourceId) expanded = null;
      delete resultsBySession[sourceId];
      await loadSessions();
    } catch (e) {
      console.error('Failed to merge sessions:', e);
      alert('Failed to merge sessions');
    }
  }

  function flaggedCount(results: LabResult[]): number {
    return results.filter((r) => r.flag && r.flag !== 'N').length;
  }

  function flaggedSeverity(results: LabResult[]): 'high' | 'low' | 'mixed' | 'none' {
    const flags = results.filter((r) => r.flag && r.flag !== 'N').map((r) => r.flag);
    if (flags.length === 0) return 'none';
    const hi = flags.some((f) => f === 'H' || f === 'HH');
    const lo = flags.some((f) => f === 'L' || f === 'LL');
    if (hi && lo) return 'mixed';
    return hi ? 'high' : 'low';
  }
</script>

<div class="results-view">
  <DiagnosisAccordion {onNavigate} />

  <div class="results-header">
    <h1>Lab Results</h1>
    <div class="header-actions">
      <button class="configure-btn" onclick={openLabConfig}>Configure</button>
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
    {#each sessions as session (session.id)}
      {@const isExpanded = expanded === session.id}
      {@const results = session.id !== null ? (resultsBySession[session.id] ?? []) : []}
      {@const siblingIds = session.test_date && session.id ? (duplicateDateMap.get(session.test_date) ?? []).filter(id => id !== session.id) : []}
      <article class="session-card" class:expanded={isExpanded} class:duplicate={siblingIds.length > 0}>
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <header class="session-header" onclick={() => toggle(session.id)}>
          <span class="session-date">{formatDate(session.test_date)}</span>
          <span class="session-lab">{session.lab_name || '--'}</span>
          <span class="session-panel">{session.notes || ''}</span>
          {#if isExpanded && flaggedCount(results) > 0}
            <span class="flagged-pill" data-severity={flaggedSeverity(results)}>{flaggedCount(results)} flagged</span>
          {:else if isExpanded}
            <span class="flagged-pill" data-severity="none">all normal</span>
          {/if}
          {#if siblingIds.length > 0}
            <button class="merge-btn" title="Merge into earliest session for this date"
              onclick={(e) => { e.stopPropagation(); session.id && handleMerge(siblingIds[0], session.id); }}>
              Merge
            </button>
          {/if}
          <button
            class="edit-btn"
            onclick={(e) => { e.stopPropagation(); onNavigate('lab-entry', session.id); }}
          >Edit</button>
          <button
            class="delete-btn"
            onclick={(e) => { e.stopPropagation(); session.id && handleDelete(session.id); }}
          >Delete</button>
        </header>

        {#if isExpanded}
          {#if loadingDetail && results.length === 0}
            <p class="muted detail-msg">Loading...</p>
          {:else if results.length === 0}
            <p class="muted detail-msg">No results recorded for this session.</p>
          {:else}
            <table class="results-table">
              <thead>
                <tr>
                  <th>TEST</th>
                  <th>VALUE</th>
                  <th>UNIT</th>
                  <th>RANGE</th>
                  <th>FLAG</th>
                </tr>
              </thead>
              <tbody>
                {#each results as r}
                  <tr>
                    <td class="test">
                      {r.test_name}
                      <button class="info-btn" onclick={() => openGlossary(r.test_name)} title="View in glossary">i</button>
                    </td>
                    <td class="value">
                      {#if r.value != null}{r.value}{:else}{r.text_value}{/if}
                    </td>
                    <td class="unit">{r.unit}</td>
                    <td class="range">
                      {#if r.ref_range_low != null && r.ref_range_high != null}
                        {r.ref_range_low}-{r.ref_range_high}
                      {:else if r.ref_range_high != null}
                        &lt; {r.ref_range_high}
                      {:else if r.ref_range_low != null}
                        &gt; {r.ref_range_low}
                      {/if}
                    </td>
                    <td>
                      {#if r.flag && r.flag !== 'N'}
                        <span class="flag-pill" data-flag={r.flag}>{r.flag}</span>
                      {/if}
                    </td>
                  </tr>
                {/each}
              </tbody>
            </table>
          {/if}
        {/if}
      </article>
    {/each}
  {/if}
</div>

<style>
  .results-view { max-width: 1160px; margin: 0 auto; padding: 24px 8px; }

  .results-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 20px;
    gap: 12px;
  }
  .results-header h1 {
    font-family: var(--font-heading);
    font-size: 28px;
    font-weight: 900;
    margin: 0;
  }
  .header-actions { display: flex; gap: 8px; }

  .configure-btn {
    padding: 6px 12px;
    border: 1px solid var(--border);
    background: var(--surface);
    color: var(--text-muted);
    border-radius: 8px;
    cursor: pointer;
  }

  .muted { color: var(--text-muted); }

  .empty-state {
    padding: 40px;
    text-align: center;
    border: 1px dashed var(--border);
    border-radius: 12px;
    margin-top: 20px;
  }

  .session-card {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 12px;
    margin-bottom: 12px;
    overflow: hidden;
  }
  .session-card.duplicate {
    border-color: var(--accent-high);
  }

  .session-header {
    display: flex;
    align-items: center;
    gap: 16px;
    padding: 18px 20px;
    cursor: pointer;
  }
  .session-date { font-weight: 700; white-space: nowrap; }
  .session-lab { color: var(--text-muted); white-space: nowrap; }
  .session-panel {
    color: var(--text-muted);
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    min-width: 0;
  }

  .flagged-pill {
    padding: 4px 10px;
    border-radius: 999px;
    font-size: 12px;
    font-weight: 600;
    white-space: nowrap;
  }
  .flagged-pill[data-severity="high"] {
    background: color-mix(in oklab, var(--accent-high) 22%, transparent);
    color: var(--accent-high);
  }
  .flagged-pill[data-severity="low"],
  .flagged-pill[data-severity="mixed"] {
    background: color-mix(in oklab, var(--accent-low) 15%, transparent);
    color: var(--accent-low);
  }
  .flagged-pill[data-severity="none"] {
    background: color-mix(in oklab, var(--text-muted) 10%, transparent);
    color: var(--text-muted);
  }

  .edit-btn, .delete-btn, .merge-btn {
    padding: 6px 14px;
    border-radius: 8px;
    border: 1px solid var(--border);
    background: var(--surface);
    color: var(--text);
    cursor: pointer;
    font-size: 13px;
  }
  .merge-btn { color: var(--accent-high); border-color: var(--accent-high); }
  .delete-btn { color: var(--accent-low); }

  .detail-msg { padding: 12px 20px; border-top: 1px solid var(--border); margin: 0; }

  .results-table {
    width: 100%;
    border-top: 1px solid var(--border);
    border-collapse: collapse;
  }
  .results-table th {
    text-align: left;
    padding: 10px 20px;
    font-size: 11px;
    color: var(--text-muted);
    letter-spacing: 0.06em;
    font-weight: 600;
  }
  .results-table td {
    padding: 12px 20px;
    border-top: 1px solid var(--border);
  }
  .results-table td.test { font-weight: 700; }
  .results-table td.value { font-family: var(--font-mono); font-weight: 700; }
  .results-table td.unit,
  .results-table td.range { color: var(--text-muted); font-family: var(--font-mono); }

  .info-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 16px;
    height: 16px;
    border-radius: 50%;
    border: 1px solid var(--border);
    background: transparent;
    color: var(--text-muted);
    font-size: 10px;
    font-style: italic;
    cursor: pointer;
    padding: 0;
    margin-left: 6px;
  }

  .flag-pill {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 22px;
    height: 22px;
    padding: 0 6px;
    border-radius: 6px;
    font-size: 11px;
    font-weight: 700;
    color: white;
  }
  .flag-pill[data-flag="H"], .flag-pill[data-flag="HH"] { background: var(--accent-high); }
  .flag-pill[data-flag="L"], .flag-pill[data-flag="LL"] { background: var(--accent-low); }
</style>
