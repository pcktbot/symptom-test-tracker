<script lang="ts">
  import { onMount } from 'svelte';
  import { getLatestAbnormal } from '$lib/db';
  import { flagClass } from '$lib/utils';
  import type { LabResult, View } from '$lib/types';

  let { onNavigate }: { onNavigate: (view: View, sessionId?: number | null) => void } = $props();

  let abnormals: LabResult[] = $state([]);
  let loading = $state(true);

  onMount(async () => {
    try {
      abnormals = await getLatestAbnormal();
    } catch (e) {
      console.error('Failed to load abnormal results:', e);
    }
    loading = false;
  });

  function groupByPanel(results: LabResult[]): Record<string, LabResult[]> {
    const groups: Record<string, LabResult[]> = {};
    for (const r of results) {
      const panel = r.panel || 'Other';
      if (!groups[panel]) groups[panel] = [];
      groups[panel].push(r);
    }
    return groups;
  }

  let grouped = $derived(groupByPanel(abnormals));
</script>

<div class="dashboard">
  <div class="header">
    <h1>Dashboard</h1>
    <button class="primary" onclick={() => onNavigate('lab-entry')}>+ New Lab Entry</button>
  </div>

  {#if loading}
    <p class="muted">Loading...</p>
  {:else if abnormals.length === 0}
    <div class="empty-state">
      <p>No abnormal lab values found.</p>
      <p class="muted">Enter lab results to see flagged values here.</p>
    </div>
  {:else}
    <p class="summary">{abnormals.length} abnormal value{abnormals.length !== 1 ? 's' : ''} from latest results</p>

    {#each Object.entries(grouped) as [panel, results]}
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
                <td class="test-name">{r.test_name}</td>
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

<style>
  .dashboard { max-width: 900px; }

  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 20px;
  }

  .summary {
    color: var(--color-text-muted);
    margin-bottom: 16px;
    font-size: 13px;
  }

  .muted { color: var(--color-text-muted); }

  .empty-state {
    padding: 40px;
    text-align: center;
    border: 1px dashed var(--color-border);
    border-radius: var(--radius);
    margin-top: 20px;
  }

  .panel-group {
    margin-bottom: 20px;
  }

  .panel-group h3 {
    margin-bottom: 8px;
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
    padding: 6px 10px;
    color: var(--color-text-muted);
    font-weight: 500;
    font-size: 12px;
    text-transform: uppercase;
    letter-spacing: 0.03em;
    border-bottom: 1px solid var(--color-border);
  }

  td {
    padding: 6px 10px;
    border-bottom: 1px solid var(--color-border);
  }

  .test-name { font-weight: 500; }
  .value { font-family: var(--font-mono); }
  .unit { color: var(--color-text-muted); }
  .ref-range { color: var(--color-text-muted); font-family: var(--font-mono); font-size: 12px; }
</style>
