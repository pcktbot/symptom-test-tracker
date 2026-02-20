<script lang="ts">
  import { exportData } from '$lib/db';
  import { todayString } from '$lib/utils';
  import { save } from '@tauri-apps/plugin-dialog';
  import { writeTextFile } from '@tauri-apps/plugin-fs';

  const oneYearAgo = (() => {
    const d = new Date();
    d.setFullYear(d.getFullYear() - 1);
    return d.toISOString().slice(0, 10);
  })();
  let startDate = $state(oneYearAgo);
  let endDate = $state(todayString());
  let includeLabs = $state(true);
  let includeSymptoms = $state(true);
  let format: 'json' | 'csv' = $state('json');
  let exporting = $state(false);
  let exported = $state(false);

  async function handleExport() {
    exporting = true;
    try {
      const data = await exportData(startDate, endDate, includeLabs, includeSymptoms, format);

      const ext = format === 'json' ? 'json' : 'csv';
      const filePath = await save({
        defaultPath: `symptom-tracker-export.${ext}`,
        filters: [
          { name: format === 'json' ? 'JSON' : 'CSV', extensions: [ext] },
        ],
      });

      if (filePath) {
        await writeTextFile(filePath, data);
        exported = true;
        setTimeout(() => exported = false, 3000);
      }
    } catch (e) {
      console.error('Export failed:', e);
      alert('Export failed: ' + e);
    }
    exporting = false;
  }
</script>

<div class="export-view">
  <h1>Export Data</h1>

  <div class="form">
    <div class="form-row">
      <div class="field">
        <label for="start-date">Start Date</label>
        <input id="start-date" type="date" bind:value={startDate} />
      </div>
      <div class="field">
        <label for="end-date">End Date</label>
        <input id="end-date" type="date" bind:value={endDate} />
      </div>
    </div>

    <div class="form-row">
      <label class="checkbox-label">
        <input type="checkbox" bind:checked={includeLabs} />
        Include Lab Results
      </label>
      <label class="checkbox-label">
        <input type="checkbox" bind:checked={includeSymptoms} />
        Include Symptom Logs
      </label>
    </div>

    <div class="form-row">
      <div class="field">
        <label for="format">Format</label>
        <select id="format" bind:value={format}>
          <option value="json">JSON</option>
          <option value="csv">CSV</option>
        </select>
      </div>
    </div>

    <div class="form-row">
      <button class="primary" onclick={handleExport} disabled={exporting || (!includeLabs && !includeSymptoms)}>
        {exporting ? 'Exporting...' : 'Export'}
      </button>
      {#if exported}
        <span class="export-success">Export saved successfully</span>
      {/if}
    </div>
  </div>
</div>

<style>
  .export-view { max-width: 500px; }

  h1 { margin-bottom: 20px; }

  .form {
    display: flex;
    flex-direction: column;
    gap: 16px;
    padding: 16px;
    border: 1px solid var(--color-border);
    border-radius: var(--radius);
  }

  .form-row {
    display: flex;
    gap: 12px;
    align-items: flex-end;
    flex-wrap: wrap;
  }

  .field {
    display: flex;
    flex-direction: column;
  }

  .checkbox-label {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 14px;
    color: var(--color-text);
    cursor: pointer;
  }

  .checkbox-label input[type="checkbox"] {
    margin: 0;
  }

  .export-success {
    color: var(--color-success);
    font-size: 13px;
    font-weight: 500;
  }
</style>
