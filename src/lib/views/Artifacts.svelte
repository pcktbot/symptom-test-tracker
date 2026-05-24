<script lang="ts">
  import { open } from '@tauri-apps/plugin-dialog';
  import { getArtifacts, saveArtifactPaste, saveArtifactFile, deleteArtifact, extractLabsFromArtifact, saveLabSession } from '$lib/db';
  import DiagnosisAccordion from '$lib/components/DiagnosisAccordion.svelte';
  import type { Artifact, ExtractionResult, LabSession, LabResult, View } from '$lib/types';

  let { onNavigate, openExport }: {
    onNavigate: (view: View) => void;
    openExport: () => void;
  } = $props();

  let artifacts = $state<Artifact[]>([]);
  let showAdd = $state(false);
  let addTab = $state<'paste' | 'file'>('paste');
  let addTitle = $state('');
  let addContent = $state('');
  let saving = $state(false);
  let extracting = $state<number | null>(null);
  let extraction = $state<ExtractionResult | null>(null);
  let extractionArtifactId = $state<number | null>(null);
  let extractionDate = $state('');
  let importing = $state(false);

  $effect(() => { getArtifacts().then(a => artifacts = a); });

  function resetAdd() {
    showAdd = false; addTitle = ''; addContent = ''; addTab = 'paste';
  }

  async function handlePickFile() {
    const path = await open({ multiple: false, filters: [{ name: 'Documents', extensions: ['pdf', 'txt', 'html', 'htm'] }] });
    if (!path || typeof path !== 'string') return;
    if (!addTitle) {
      addTitle = path.split('/').pop()?.replace(/\.[^.]+$/, '') ?? '';
    }
    saving = true;
    try {
      const id = await saveArtifactFile(addTitle || 'Untitled', path);
      artifacts = await getArtifacts();
      resetAdd();
      await startExtraction(id);
    } finally { saving = false; }
  }

  async function handleSavePaste() {
    if (!addContent.trim()) return;
    saving = true;
    try {
      const id = await saveArtifactPaste(addTitle || 'Untitled', addContent);
      artifacts = await getArtifacts();
      resetAdd();
      await startExtraction(id);
    } finally { saving = false; }
  }

  async function startExtraction(id: number) {
    extracting = id;
    extractionArtifactId = id;
    try {
      const result = await extractLabsFromArtifact(id);
      extraction = result;
      extractionDate = result.date ?? new Date().toISOString().slice(0, 10);
    } catch (e) {
      extraction = null;
    } finally { extracting = null; }
  }

  async function handleImport() {
    if (!extraction || !extractionDate) return;
    importing = true;
    try {
      const session: LabSession = { id: null, test_date: extractionDate, lab_name: '', notes: 'Imported from artifact' };
      const results = extraction.results.map(r => ({
        id: null, session_id: null,
        test_name: r.test_name, panel: r.panel,
        value: r.value, text_value: r.text_value,
        unit: r.unit, ref_range_low: r.ref_range_low,
        ref_range_high: r.ref_range_high, flag: r.flag,
      }));
      // LabResult fields match the mapped shape; id/session_id are nullable as expected
      await saveLabSession(session, results as LabResult[]);
      extraction = null; extractionArtifactId = null;
    } finally { importing = false; }
  }

  function dismissExtraction() { extraction = null; extractionArtifactId = null; }

  async function handleDelete(id: number) {
    await deleteArtifact(id);
    artifacts = await getArtifacts();
  }

  function typeLabel(ct: string) {
    return ct === 'pdf' ? 'PDF' : ct === 'html' ? 'HTML' : 'Text';
  }
  function typeClass(ct: string) {
    return ct === 'pdf' ? 'badge-pdf' : ct === 'html' ? 'badge-html' : 'badge-text';
  }
</script>

<div class="view">
  <DiagnosisAccordion {onNavigate} />

  <div class="view-header">
    <h1>Artifacts</h1>
    <div class="header-actions">
      <button class="configure-btn" onclick={openExport}>
        <svg width="14" height="14" viewBox="0 0 16 16" fill="currentColor">
          <path d="M.5 9.9a.5.5 0 0 1 .5.5v2.5a1 1 0 0 0 1 1h12a1 1 0 0 0 1-1v-2.5a.5.5 0 0 1 1 0v2.5a2 2 0 0 1-2 2H2a2 2 0 0 1-2-2v-2.5a.5.5 0 0 1 .5-.5z"/>
          <path d="M7.646 11.854a.5.5 0 0 0 .708 0l3-3a.5.5 0 0 0-.708-.708L8.5 10.293V1.5a.5.5 0 0 0-1 0v8.793L5.354 8.146a.5.5 0 1 0-.708.708l3 3z"/>
        </svg>
        Export
      </button>
      <button class="btn-primary" onclick={() => showAdd = true}>+ Add</button>
    </div>
  </div>

  {#if showAdd}
    <div class="add-card">
      <div class="field" style="margin-bottom:12px">
        <label>Title</label>
        <input bind:value={addTitle} placeholder="e.g. Rheumatology Follow-up — Mar 2025" />
      </div>
      <div class="tabs">
        <button class="tab" class:active={addTab === 'paste'} onclick={() => addTab = 'paste'}>Paste text</button>
        <button class="tab" class:active={addTab === 'file'} onclick={() => addTab = 'file'}>Upload file</button>
      </div>
      {#if addTab === 'paste'}
        <textarea bind:value={addContent} rows="8" placeholder="Paste your after-visit summary, lab report, or any medical document text here…"></textarea>
        <div class="form-actions">
          <button class="btn-ghost" onclick={resetAdd}>Cancel</button>
          <button class="btn-primary" onclick={handleSavePaste} disabled={saving || !addContent.trim()}>
            {saving ? 'Saving…' : 'Save & Extract'}
          </button>
        </div>
      {:else}
        <div class="dropzone" onclick={handlePickFile} role="button" tabindex="0" onkeydown={e => e.key === 'Enter' && handlePickFile()}>
          <div class="dropzone-icon">📄</div>
          <div class="dropzone-text">Click to choose a file</div>
          <div class="dropzone-sub">PDF, plain text, or HTML</div>
        </div>
        <div class="form-actions">
          <button class="btn-ghost" onclick={resetAdd}>Cancel</button>
        </div>
      {/if}
    </div>
  {/if}

  {#if extracting !== null}
    <div class="extraction-card extracting">
      <div class="extraction-status">
        <span class="spinner"></span> Extracting lab results…
      </div>
    </div>
  {/if}

  {#if extraction !== null}
    <div class="extraction-card">
      <div class="extraction-header">
        <span>Extracted <strong>{extraction.results.length}</strong> lab result{extraction.results.length !== 1 ? 's' : ''}.</span>
        <div class="date-row">
          <label>Lab date:</label>
          <input type="date" bind:value={extractionDate} />
        </div>
      </div>
      <div class="results-table">
        {#each extraction.results.slice(0, 8) as r}
          <div class="result-row">
            <span class="result-name">{r.test_name}</span>
            <span class="result-value">{r.value ?? r.text_value} {r.unit}</span>
            <span class="result-range">{r.ref_range_low ?? ''}–{r.ref_range_high ?? ''}</span>
            <span class="flag flag-{r.flag.toLowerCase()}">{r.flag}</span>
          </div>
        {/each}
        {#if extraction.results.length > 8}
          <div class="more-results">+{extraction.results.length - 8} more</div>
        {/if}
      </div>
      <div class="form-actions">
        <button class="btn-ghost" onclick={dismissExtraction}>Dismiss</button>
        <button class="btn-primary" onclick={handleImport} disabled={importing || !extractionDate}>
          {importing ? 'Importing…' : 'Import as Lab Session →'}
        </button>
      </div>
    </div>
  {/if}

  {#if artifacts.length === 0 && !showAdd}
    <p class="empty">No artifacts yet. Add an after-visit summary or lab report to get started.</p>
  {/if}

  <div class="artifact-list">
    {#each artifacts as a (a.id)}
      <div class="artifact-row">
        <div class="artifact-info">
          <div class="artifact-title">{a.title}</div>
          <div class="artifact-meta">
            <span class="type-badge {typeClass(a.content_type)}">{typeLabel(a.content_type)}</span>
            <span class="meta-text">{a.created_at.slice(0, 10)}</span>
          </div>
        </div>
        <div class="artifact-actions">
          <button class="btn-ghost btn-sm" onclick={() => startExtraction(Number(a.id))}>Re-extract</button>
          <button class="btn-ghost btn-sm btn-danger" onclick={() => a.id && handleDelete(Number(a.id))}>Delete</button>
        </div>
      </div>
    {/each}
  </div>
</div>

<style>
  .view { max-width: 800px; }
  .view-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 20px; }
  .view-header h1 { margin: 0; }

  .header-actions {
    display: flex;
    align-items: center;
    gap: 8px;
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
  .add-card { background: var(--color-surface-raised); border: 1px solid var(--color-border); border-radius: 8px; padding: 16px; margin-bottom: 20px; }
  .field label { font-size: 12px; font-weight: 500; color: var(--color-text-muted); display: block; margin-bottom: 4px; }
  .field input { width: 100%; background: var(--color-surface); border: 1px solid var(--color-border); border-radius: var(--radius); padding: 6px 10px; font-size: 13px; color: var(--color-text); box-sizing: border-box; }
  .tabs { display: flex; border-bottom: 1px solid var(--color-border); margin-bottom: 12px; }
  .tab { background: none; border: none; border-bottom: 2px solid transparent; padding: 6px 14px; font-size: 13px; color: var(--color-text-muted); cursor: pointer; margin-bottom: -1px; }
  .tab.active { color: var(--color-accent); border-bottom-color: var(--color-accent); }
  textarea { width: 100%; background: var(--color-surface); border: 1px solid var(--color-border); border-radius: var(--radius); padding: 8px 10px; font-size: 13px; color: var(--color-text); resize: vertical; box-sizing: border-box; }
  .dropzone { border: 2px dashed var(--color-border); border-radius: 8px; padding: 32px; text-align: center; cursor: pointer; }
  .dropzone:hover { border-color: var(--color-accent); }
  .dropzone-icon { font-size: 28px; margin-bottom: 8px; }
  .dropzone-text { font-size: 13px; color: var(--color-text); margin-bottom: 4px; }
  .dropzone-sub { font-size: 11px; color: var(--color-text-muted); }
  .form-actions { display: flex; justify-content: flex-end; gap: 8px; margin-top: 12px; }
  .extraction-card { background: var(--color-surface-raised); border: 1px solid var(--color-border); border-radius: 8px; padding: 14px 16px; margin-bottom: 20px; }
  .extracting { display: flex; align-items: center; gap: 10px; }
  .extraction-status { display: flex; align-items: center; gap: 8px; font-size: 13px; color: var(--color-text-muted); }
  .spinner { display: inline-block; width: 14px; height: 14px; border: 2px solid var(--color-border); border-top-color: var(--color-accent); border-radius: 50%; animation: spin 0.8s linear infinite; }
  @keyframes spin { to { transform: rotate(360deg); } }
  .extraction-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 10px; font-size: 13px; }
  .date-row { display: flex; align-items: center; gap: 6px; font-size: 12px; color: var(--color-text-muted); }
  .date-row input { border: 1px solid var(--color-border); border-radius: 4px; padding: 3px 6px; font-size: 12px; background: var(--color-surface); color: var(--color-text); }
  .results-table { display: flex; flex-direction: column; gap: 3px; margin-bottom: 4px; }
  .result-row { display: grid; grid-template-columns: 1fr 100px 100px 40px; gap: 8px; font-size: 12px; padding: 4px 8px; background: var(--color-surface); border-radius: 3px; }
  .result-name { font-weight: 500; }
  .result-value, .result-range { color: var(--color-text-muted); }
  .flag { font-size: 11px; font-weight: 600; text-align: right; }
  .flag-n { color: var(--color-text-muted); }
  .flag-h, .flag-hh { color: #e05555; }
  .flag-l, .flag-ll { color: #e09055; }
  .more-results { font-size: 11px; color: var(--color-text-muted); text-align: center; padding: 4px; }
  .empty { color: var(--color-text-muted); font-size: 14px; }
  .artifact-list { display: flex; flex-direction: column; gap: 6px; }
  .artifact-row { display: flex; align-items: center; justify-content: space-between; background: var(--color-surface-raised); border: 1px solid var(--color-border); border-radius: 6px; padding: 10px 14px; }
  .artifact-title { font-size: 13px; font-weight: 500; margin-bottom: 4px; }
  .artifact-meta { display: flex; align-items: center; gap: 8px; }
  .type-badge { border-radius: 3px; padding: 1px 6px; font-size: 10px; font-weight: 500; }
  .badge-pdf { background: #1a2a3a; color: #7c9cff; }
  .badge-html { background: #2a1a3a; color: #cc88ff; }
  .badge-text { background: #1a3a2a; color: #6bff8e; }
  .meta-text { font-size: 11px; color: var(--color-text-muted); }
  .artifact-actions { display: flex; gap: 6px; }
  .btn-sm { padding: 3px 10px; font-size: 12px; }
  .btn-danger { color: var(--color-error, #e05555); }
</style>
