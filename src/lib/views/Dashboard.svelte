<script lang="ts">
  import { onMount } from 'svelte';
  import { getLatestAbnormalWithPrevious } from '$lib/db';
  import { flagClass } from '$lib/utils';
  import type { AbnormalResult, View } from '$lib/types';

  let { onNavigate, openGlossary }: { onNavigate: (view: View, sessionId?: number | null) => void; openGlossary: (testName?: string) => void } = $props();

  let abnormals: AbnormalResult[] = $state([]);
  let loading = $state(true);

  onMount(async () => {
    try {
      abnormals = await getLatestAbnormalWithPrevious();
    } catch (e) {
      console.error('Failed to load abnormal results:', e);
    }
    loading = false;
  });

  function groupByPanel(results: AbnormalResult[]): Record<string, AbnormalResult[]> {
    const groups: Record<string, AbnormalResult[]> = {};
    for (const r of results) {
      const panel = r.panel || 'Other';
      if (!groups[panel]) groups[panel] = [];
      groups[panel].push(r);
    }
    return groups;
  }

  function changeInfo(r: AbnormalResult): { text: string; cssClass: string } {
    if (r.value == null || r.prev_value == null) {
      return { text: '--', cssClass: 'change-neutral' };
    }
    const delta = r.value - r.prev_value;
    if (Math.abs(delta) < 0.001) {
      return { text: '0', cssClass: 'change-neutral' };
    }

    const refLow = r.ref_range_low;
    const refHigh = r.ref_range_high;

    // Determine if moving closer to or further from the reference range midpoint
    let improving = false;
    if (refLow != null && refHigh != null) {
      const mid = (refLow + refHigh) / 2;
      improving = Math.abs(r.value - mid) < Math.abs(r.prev_value - mid);
    } else if (refHigh != null) {
      // Only upper bound — lower is better
      improving = r.value < r.prev_value;
    } else if (refLow != null) {
      // Only lower bound — higher is better
      improving = r.value > r.prev_value;
    }

    const sign = delta > 0 ? '+' : '';
    const text = `${sign}${Number.isInteger(delta) ? delta : delta.toFixed(1)}`;
    const cssClass = improving ? 'change-improving' : 'change-worsening';
    return { text, cssClass };
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
              <th>Previous</th>
              <th>Change</th>
              <th>Unit</th>
              <th>Reference Range</th>
              <th>Flag</th>
            </tr>
          </thead>
          <tbody>
            {#each results as r}
              {@const change = changeInfo(r)}
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
                <td class="prev-value">
                  {#if r.prev_value != null}
                    {r.prev_value}
                  {:else if r.prev_text_value}
                    {r.prev_text_value}
                  {:else}
                    <span class="muted">--</span>
                  {/if}
                </td>
                <td class="change-cell {change.cssClass}">{change.text}</td>
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
  .dashboard { max-width: 960px; }

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

  .test-name {
    font-weight: 500;
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .value { font-family: var(--font-mono); }
  .prev-value { font-family: var(--font-mono); color: var(--color-text-muted); font-size: 12px; }
  .unit { color: var(--color-text-muted); }
  .ref-range { color: var(--color-text-muted); font-family: var(--font-mono); font-size: 12px; }

  .change-cell {
    font-family: var(--font-mono);
    font-size: 12px;
    font-weight: 600;
  }

  .change-improving { color: var(--color-success); }
  .change-worsening { color: var(--color-danger); }
  .change-neutral { color: var(--color-text-muted); font-weight: 400; }
</style>
