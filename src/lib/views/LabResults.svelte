<script lang="ts">
  import { onMount } from 'svelte';
  import { getLabSessions, deleteLabSession } from '$lib/db';
  import { formatDate } from '$lib/utils';
  import type { LabSession, View } from '$lib/types';

  let { onNavigate }: { onNavigate: (view: View, sessionId?: number | null) => void } = $props();

  let sessions: LabSession[] = $state([]);
  let loading = $state(true);

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

  async function handleDelete(id: number) {
    if (!confirm('Delete this lab session and all its results?')) return;
    try {
      await deleteLabSession(id);
      await loadSessions();
    } catch (e) {
      console.error('Failed to delete session:', e);
    }
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
    <table>
      <thead>
        <tr>
          <th>Date</th>
          <th>Lab</th>
          <th>Notes</th>
          <th></th>
        </tr>
      </thead>
      <tbody>
        {#each sessions as session}
          <tr>
            <td class="date">{formatDate(session.test_date)}</td>
            <td>{session.lab_name || '--'}</td>
            <td class="notes">{session.notes || '--'}</td>
            <td class="actions">
              <button onclick={() => onNavigate('lab-entry', session.id)}>Edit</button>
              <button class="danger" onclick={() => session.id && handleDelete(session.id)}>Delete</button>
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
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

  table {
    width: 100%;
    border-collapse: collapse;
  }

  th {
    text-align: left;
    padding: 8px 10px;
    color: var(--color-text-muted);
    font-weight: 500;
    font-size: 12px;
    text-transform: uppercase;
    letter-spacing: 0.03em;
    border-bottom: 1px solid var(--color-border);
  }

  td {
    padding: 8px 10px;
    border-bottom: 1px solid var(--color-border);
  }

  .date { font-weight: 500; white-space: nowrap; }
  .notes { color: var(--color-text-muted); max-width: 300px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }

  .actions {
    text-align: right;
    white-space: nowrap;
  }

  .actions button {
    font-size: 12px;
    padding: 3px 8px;
    margin-left: 4px;
  }
</style>
