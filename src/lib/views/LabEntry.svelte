<script lang="ts">
  import { onMount } from 'svelte';
  import { getLabSession, saveLabSession } from '$lib/db';
  import { todayString, getMergedPanels } from '$lib/utils';
  import type { LabSession, LabResult, PanelDefinition, View } from '$lib/types';

  let { sessionId = null, onNavigate, openGlossary }: { sessionId: number | null; onNavigate: (view: View) => void; openGlossary: (testName?: string) => void } = $props();

  let testDate = $state(todayString());
  let labName = $state('');
  let notes = $state('');
  let saving = $state(false);
  let expandedPanels: Record<string, boolean> = $state({});
  let panels: PanelDefinition[] = $state([]);

  // Store result values keyed by test name
  let resultValues: Record<string, {
    value: string;
    text_value: string;
    flag: string;
    ref_low: string;
    ref_high: string;
    unit: string;
    panel: string;
    text_only: boolean;
  }> = $state({});

  onMount(async () => {
    // Load panels (built-in + custom)
    panels = await getMergedPanels();

    // Initialize all test fields with defaults from panel definitions
    for (const panel of panels) {
      for (const test of panel.tests) {
        resultValues[test.name] = {
          value: '',
          text_value: '',
          flag: 'N',
          ref_low: test.ref_low != null ? String(test.ref_low) : '',
          ref_high: test.ref_high != null ? String(test.ref_high) : '',
          unit: test.unit,
          panel: panel.name,
          text_only: test.text_only ?? false,
        };
      }
    }

    if (sessionId) {
      try {
        const data = await getLabSession(sessionId);
        testDate = data.session.test_date;
        labName = data.session.lab_name;
        notes = data.session.notes;

        for (const r of data.results) {
          if (resultValues[r.test_name]) {
            resultValues[r.test_name].value = r.value != null ? String(r.value) : '';
            resultValues[r.test_name].text_value = r.text_value;
            resultValues[r.test_name].flag = r.flag;
            resultValues[r.test_name].ref_low = r.ref_range_low != null ? String(r.ref_range_low) : '';
            resultValues[r.test_name].ref_high = r.ref_range_high != null ? String(r.ref_range_high) : '';
          }
          // Expand panels that have data
          if (r.panel) expandedPanels[r.panel] = true;
        }
      } catch (e) {
        console.error('Failed to load session:', e);
      }
    }
  });

  function togglePanel(name: string) {
    expandedPanels[name] = !expandedPanels[name];
  }

  function autoFlag(testName: string) {
    const r = resultValues[testName];
    if (!r || r.text_only || !r.value) {
      if (r) r.flag = 'N';
      return;
    }
    const val = parseFloat(r.value);
    const lo = r.ref_low ? parseFloat(r.ref_low) : null;
    const hi = r.ref_high ? parseFloat(r.ref_high) : null;

    if (isNaN(val)) { r.flag = 'N'; return; }

    if (hi != null && val > hi) {
      // Check if critically high (more than 2x above upper limit)
      r.flag = (lo != null && hi != null && val > hi + (hi - lo)) ? 'HH' : 'H';
    } else if (lo != null && val < lo) {
      r.flag = (lo != null && hi != null && val < lo - (hi - lo)) ? 'LL' : 'L';
    } else {
      r.flag = 'N';
    }
  }

  async function handleSave() {
    saving = true;
    try {
      const session: LabSession = {
        id: sessionId,
        test_date: testDate,
        lab_name: labName,
        notes,
      };

      const results: LabResult[] = [];
      for (const [testName, r] of Object.entries(resultValues)) {
        results.push({
          id: null,
          session_id: null,
          test_name: testName,
          panel: r.panel,
          value: r.value ? parseFloat(r.value) : null,
          text_value: r.text_value,
          unit: r.unit,
          ref_range_low: r.ref_low ? parseFloat(r.ref_low) : null,
          ref_range_high: r.ref_high ? parseFloat(r.ref_high) : null,
          flag: r.flag,
        });
      }

      await saveLabSession(session, results);
      onNavigate('lab-results');
    } catch (e) {
      console.error('Failed to save:', e);
      alert('Failed to save lab session');
    }
    saving = false;
  }
</script>

<div class="lab-entry">
  <div class="header">
    <h1>{sessionId ? 'Edit' : 'New'} Lab Entry</h1>
    <div class="header-actions">
      <button onclick={() => onNavigate('lab-results')}>Cancel</button>
      <button class="primary" onclick={handleSave} disabled={saving}>
        {saving ? 'Saving...' : 'Save'}
      </button>
    </div>
  </div>

  <div class="session-meta">
    <div class="field">
      <label for="test-date">Test Date</label>
      <input id="test-date" type="date" bind:value={testDate} />
    </div>
    <div class="field">
      <label for="lab-name">Lab Name</label>
      <input id="lab-name" type="text" bind:value={labName} placeholder="e.g. Quest, LabCorp" />
    </div>
    <div class="field wide">
      <label for="session-notes">Notes</label>
      <input id="session-notes" type="text" bind:value={notes} placeholder="Optional notes" />
    </div>
  </div>

  <div class="panels">
    {#each panels as panel}
      <div class="panel">
        <button class="panel-header" onclick={() => togglePanel(panel.name)}>
          <span class="panel-toggle">{expandedPanels[panel.name] ? '-' : '+'}</span>
          <span class="panel-name">{panel.name}</span>
          <span class="panel-count">
            {panel.tests.filter(t => {
              const r = resultValues[t.name];
              return r && (r.value || r.text_value);
            }).length} / {panel.tests.length}
          </span>
        </button>

        {#if expandedPanels[panel.name]}
          <div class="panel-body">
            <table>
              <thead>
                <tr>
                  <th class="col-name">Test</th>
                  <th class="col-value">Value</th>
                  <th class="col-unit">Unit</th>
                  <th class="col-ref">Ref Low</th>
                  <th class="col-ref">Ref High</th>
                  <th class="col-flag">Flag</th>
                </tr>
              </thead>
              <tbody>
                {#each panel.tests as test}
                  {@const r = resultValues[test.name]}
                  {#if r}
                    <tr>
                      <td class="test-name">
                        {test.name}
                        <button class="info-btn" onclick={() => openGlossary(test.name)} title="View in glossary">i</button>
                      </td>
                      <td>
                        {#if r.text_only}
                          <input
                            type="text"
                            class="input-sm"
                            bind:value={r.text_value}
                            placeholder="e.g. 1:320 Speckled"
                          />
                        {:else}
                          <input
                            type="number"
                            step="any"
                            class="input-sm mono"
                            bind:value={r.value}
                            oninput={() => autoFlag(test.name)}
                          />
                        {/if}
                      </td>
                      <td class="unit">{r.unit}</td>
                      <td>
                        {#if !r.text_only}
                          <input type="number" step="any" class="input-xs mono" bind:value={r.ref_low} />
                        {/if}
                      </td>
                      <td>
                        {#if !r.text_only}
                          <input type="number" step="any" class="input-xs mono" bind:value={r.ref_high} />
                        {/if}
                      </td>
                      <td>
                        <select class="input-xs" bind:value={r.flag}>
                          <option value="N">N</option>
                          <option value="L">L</option>
                          <option value="H">H</option>
                          <option value="LL">LL</option>
                          <option value="HH">HH</option>
                        </select>
                      </td>
                    </tr>
                  {/if}
                {/each}
              </tbody>
            </table>
          </div>
        {/if}
      </div>
    {/each}
  </div>
</div>

<style>
  .lab-entry { max-width: 960px; }

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

  .session-meta {
    display: flex;
    gap: 12px;
    margin-bottom: 24px;
    flex-wrap: wrap;
  }

  .field {
    display: flex;
    flex-direction: column;
  }

  .field.wide {
    flex: 1;
    min-width: 200px;
  }

  .field input {
    min-width: 160px;
  }

  .panels {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .panel {
    border: 1px solid var(--color-border);
    border-radius: var(--radius);
  }

  .panel-header {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 10px 14px;
    border: none;
    background: var(--color-surface-raised);
    border-radius: var(--radius);
    font-weight: 500;
    cursor: pointer;
    text-align: left;
  }

  .panel-header:hover {
    background: var(--color-border);
  }

  .panel-toggle {
    font-family: var(--font-mono);
    width: 14px;
    color: var(--color-text-muted);
  }

  .panel-name { flex: 1; }

  .panel-count {
    font-size: 12px;
    color: var(--color-text-muted);
    font-weight: 400;
  }

  .panel-body {
    padding: 8px;
  }

  table {
    width: 100%;
    border-collapse: collapse;
  }

  th {
    text-align: left;
    padding: 4px 6px;
    color: var(--color-text-muted);
    font-size: 11px;
    font-weight: 500;
    text-transform: uppercase;
    letter-spacing: 0.03em;
    border-bottom: 1px solid var(--color-border);
  }

  td {
    padding: 3px 6px;
    border-bottom: 1px solid var(--color-border);
    vertical-align: middle;
  }

  .test-name {
    font-weight: 500;
    font-size: 13px;
    white-space: nowrap;
    display: flex;
    align-items: center;
    gap: 5px;
  }

  .unit {
    color: var(--color-text-muted);
    font-size: 12px;
  }

  .col-name { width: 180px; }
  .col-value { width: 140px; }
  .col-unit { width: 70px; }
  .col-ref { width: 80px; }
  .col-flag { width: 60px; }

  .input-sm {
    width: 120px;
    padding: 3px 6px;
    font-size: 13px;
  }

  .input-xs {
    width: 70px;
    padding: 3px 6px;
    font-size: 12px;
  }

  .mono {
    font-family: var(--font-mono);
  }
</style>
