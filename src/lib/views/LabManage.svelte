<script lang="ts">
  import { onMount } from 'svelte';
  import { getCustomLabTests, saveCustomLabTest, deleteCustomLabTest } from '$lib/db';
  import { LAB_PANELS } from '$lib/utils';
  import type { CustomLabTest } from '$lib/types';

  let customTests: CustomLabTest[] = $state([]);
  let loading = $state(true);
  let saving = $state(false);
  let editingId: number | null = $state(null);

  // Form fields
  let formName = $state('');
  let formPanel = $state('CBC');
  let formUnit = $state('');
  let formRefLow = $state('');
  let formRefHigh = $state('');
  let formTextOnly = $state(false);
  let formDescription = $state('');
  let formClinical = $state('');
  let formError = $state('');

  const panelNames = LAB_PANELS.map(p => p.name);

  onMount(async () => {
    await loadCustomTests();
  });

  async function loadCustomTests() {
    loading = true;
    try {
      customTests = await getCustomLabTests();
    } catch (e) {
      console.error('Failed to load custom tests:', e);
    }
    loading = false;
  }

  function resetForm() {
    editingId = null;
    formName = '';
    formPanel = 'CBC';
    formUnit = '';
    formRefLow = '';
    formRefHigh = '';
    formTextOnly = false;
    formDescription = '';
    formClinical = '';
    formError = '';
  }

  function editTest(t: CustomLabTest) {
    editingId = t.id;
    formName = t.name;
    formPanel = t.panel;
    formUnit = t.unit;
    formRefLow = t.ref_low != null ? String(t.ref_low) : '';
    formRefHigh = t.ref_high != null ? String(t.ref_high) : '';
    formTextOnly = t.text_only;
    formDescription = t.description;
    formClinical = t.clinical;
    formError = '';
  }

  async function handleSave() {
    if (!formName.trim()) {
      formError = 'Name is required';
      return;
    }
    saving = true;
    formError = '';
    try {
      await saveCustomLabTest({
        id: editingId,
        name: formName.trim(),
        panel: formPanel,
        unit: formUnit.trim(),
        ref_low: formRefLow ? parseFloat(formRefLow) : null,
        ref_high: formRefHigh ? parseFloat(formRefHigh) : null,
        text_only: formTextOnly,
        description: formDescription.trim(),
        clinical: formClinical.trim(),
      });
      resetForm();
      await loadCustomTests();
    } catch (e: any) {
      formError = e?.toString() || 'Failed to save';
    }
    saving = false;
  }

  async function handleDelete(id: number) {
    if (!confirm('Delete this custom test?')) return;
    try {
      await deleteCustomLabTest(id);
      if (editingId === id) resetForm();
      await loadCustomTests();
    } catch (e) {
      console.error('Failed to delete:', e);
    }
  }
</script>

<div class="lab-manage">
  <h1>Manage Lab Tests</h1>
  <p class="subtitle">Built-in tests are read-only. Add custom tests to any panel below.</p>

  <div class="form-section">
    <h2>{editingId ? 'Edit' : 'Add'} Custom Test</h2>
    <div class="form-grid">
      <div class="field">
        <label for="ct-name">Test Name</label>
        <input id="ct-name" type="text" bind:value={formName} placeholder="e.g. Vitamin D" />
      </div>
      <div class="field">
        <label for="ct-panel">Panel</label>
        <select id="ct-panel" bind:value={formPanel}>
          {#each panelNames as p}
            <option value={p}>{p}</option>
          {/each}
        </select>
      </div>
      <div class="field">
        <label for="ct-unit">Unit</label>
        <input id="ct-unit" type="text" bind:value={formUnit} placeholder="e.g. ng/mL" />
      </div>
      <div class="field">
        <label for="ct-ref-low">Ref Low</label>
        <input id="ct-ref-low" type="number" step="any" bind:value={formRefLow} />
      </div>
      <div class="field">
        <label for="ct-ref-high">Ref High</label>
        <input id="ct-ref-high" type="number" step="any" bind:value={formRefHigh} />
      </div>
      <div class="field checkbox-field">
        <label>
          <input type="checkbox" bind:checked={formTextOnly} />
          Text only (no numeric value)
        </label>
      </div>
      <div class="field wide">
        <label for="ct-desc">Description</label>
        <input id="ct-desc" type="text" bind:value={formDescription} placeholder="What this test measures..." />
      </div>
      <div class="field wide">
        <label for="ct-clinical">Clinical Significance</label>
        <input id="ct-clinical" type="text" bind:value={formClinical} placeholder="Why it matters for your condition..." />
      </div>
    </div>
    {#if formError}
      <p class="error">{formError}</p>
    {/if}
    <div class="form-actions">
      {#if editingId}
        <button onclick={resetForm}>Cancel</button>
      {/if}
      <button class="primary" onclick={handleSave} disabled={saving}>
        {saving ? 'Saving...' : editingId ? 'Update' : 'Add Test'}
      </button>
    </div>
  </div>

  {#if loading}
    <p class="muted">Loading...</p>
  {:else}
    {#if customTests.length > 0}
      <div class="section">
        <h2>Custom Tests</h2>
        <table>
          <thead>
            <tr>
              <th>Test</th>
              <th>Panel</th>
              <th>Unit</th>
              <th>Ref Range</th>
              <th>Type</th>
              <th></th>
            </tr>
          </thead>
          <tbody>
            {#each customTests as t}
              <tr>
                <td class="test-name">{t.name}</td>
                <td>{t.panel}</td>
                <td class="muted">{t.unit}</td>
                <td class="mono muted">
                  {#if t.ref_low != null && t.ref_high != null}
                    {t.ref_low} - {t.ref_high}
                  {:else if t.ref_high != null}
                    &lt; {t.ref_high}
                  {:else if t.ref_low != null}
                    &gt; {t.ref_low}
                  {:else}
                    --
                  {/if}
                </td>
                <td class="muted">{t.text_only ? 'Text' : 'Numeric'}</td>
                <td class="actions">
                  <button onclick={() => editTest(t)}>Edit</button>
                  <button class="danger" onclick={() => handleDelete(t.id!)}>Delete</button>
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    {/if}

    <div class="section">
      <h2>Built-in Tests</h2>
      {#each LAB_PANELS as panel}
        <div class="builtin-panel">
          <h3>{panel.name}</h3>
          <div class="builtin-tests">
            {#each panel.tests as test}
              <span class="builtin-test">{test.name}</span>
            {/each}
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .lab-manage { max-width: 900px; }

  h1 { margin-bottom: 4px; }

  .subtitle {
    color: var(--color-text-muted);
    font-size: 13px;
    margin-bottom: 20px;
  }

  .form-section {
    border: 1px solid var(--color-border);
    border-radius: var(--radius);
    padding: 16px;
    margin-bottom: 24px;
  }

  .form-section h2 {
    margin-bottom: 12px;
  }

  .form-grid {
    display: grid;
    grid-template-columns: 1fr 1fr 1fr;
    gap: 10px;
  }

  .field {
    display: flex;
    flex-direction: column;
  }

  .field.wide {
    grid-column: 1 / -1;
  }

  .checkbox-field {
    justify-content: flex-end;
  }

  .checkbox-field label {
    display: flex;
    align-items: center;
    gap: 6px;
    cursor: pointer;
    font-size: 13px;
    color: var(--color-text);
  }

  .checkbox-field input[type="checkbox"] {
    margin: 0;
  }

  .form-actions {
    display: flex;
    gap: 8px;
    margin-top: 12px;
    justify-content: flex-end;
  }

  .error {
    color: var(--color-danger);
    font-size: 13px;
    margin-top: 8px;
  }

  .section {
    margin-bottom: 24px;
  }

  .section h2 {
    margin-bottom: 10px;
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
  .muted { color: var(--color-text-muted); }
  .mono { font-family: var(--font-mono); font-size: 12px; }

  .actions {
    display: flex;
    gap: 4px;
    justify-content: flex-end;
  }

  .actions button {
    padding: 3px 8px;
    font-size: 12px;
  }

  .builtin-panel {
    margin-bottom: 12px;
  }

  .builtin-panel h3 {
    font-size: 13px;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--color-text-muted);
    margin-bottom: 4px;
    padding-bottom: 4px;
    border-bottom: 1px solid var(--color-border);
  }

  .builtin-tests {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    padding: 4px 0;
  }

  .builtin-test {
    font-size: 12px;
    padding: 2px 8px;
    background: var(--color-surface-raised);
    border: 1px solid var(--color-border);
    border-radius: 3px;
    color: var(--color-text-muted);
  }
</style>
