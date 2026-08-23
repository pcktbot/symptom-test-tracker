<script lang="ts">
  import { getDiagnoses, saveDiagnosis, deleteDiagnosis } from '$lib/db';
  import type { Diagnosis } from '$lib/types';
  import { DEMO_APPOINTMENTS, DEMO_PROVIDERS } from '$lib/demo';

  let diagnoses = $state<Diagnosis[]>([]);
  let editing = $state<Diagnosis | null>(null);
  let saving = $state(false);
  let saveError = $state('');

  function blank(): Diagnosis {
    return {
      id: null, name: '', short_name: '', onset_date: null,
      resolution_date: null, chronic: true, source: '', details: '', created_at: ''
    };
  }

  $effect(() => {
    getDiagnoses().then(d => diagnoses = d);
  });

  function startAdd() { editing = blank(); }
  function startEdit(d: Diagnosis) { editing = { ...d }; }
  function cancelEdit() { editing = null; }

  async function handleSave() {
    if (!editing || !editing.name.trim()) return;
    saving = true;
    saveError = '';
    try {
      await saveDiagnosis(editing);
      diagnoses = await getDiagnoses();
      editing = null;
    } catch (e: any) {
      saveError = e?.toString() ?? 'Failed to save';
    } finally {
      saving = false;
    }
  }

  async function handleDelete(id: number) {
    await deleteDiagnosis(id);
    diagnoses = await getDiagnoses();
  }

  function fmtMonth(iso: string): string {
    return new Date(iso + 'T00:00').toLocaleDateString(undefined, { month: 'short' }).toUpperCase();
  }
  function fmtDay(iso: string): string {
    return String(new Date(iso + 'T00:00').getDate()).padStart(2, '0');
  }
</script>

<div class="care-team-view">
  <h1>Care Team</h1>

  <div class="two-col">
    <section class="left-col">
      <div class="col-header">
        <span class="eyebrow">DIAGNOSES</span>
        <button class="btn-primary" onclick={startAdd}>+ Add diagnosis</button>
      </div>

      {#if editing !== null}
        <div class="form-card">
          <h3>{editing.id ? 'Edit' : 'New'} diagnosis</h3>
          <div class="form-grid">
            <div class="field">
              <label>Name <span class="required">*</span></label>
              <input bind:value={editing.name} placeholder="e.g. Systemic Lupus Erythematosus" />
            </div>
            <div class="field">
              <label>Short name / abbreviation</label>
              <input bind:value={editing.short_name} placeholder="e.g. SLE" />
            </div>
            <div class="field">
              <label>Onset date</label>
              <input type="date" value={editing.onset_date ?? ''} oninput={(e) => { if (editing) editing.onset_date = (e.currentTarget as HTMLInputElement).value || null; }} />
            </div>
            <div class="field field-inline">
              <label>
                <input type="checkbox" bind:checked={editing.chronic} />
                Chronic (ongoing)
              </label>
            </div>
            {#if !editing.chronic}
              <div class="field">
                <label>Resolution date</label>
                <input type="date" value={editing.resolution_date ?? ''} oninput={(e) => { if (editing) editing.resolution_date = (e.currentTarget as HTMLInputElement).value || null; }} />
              </div>
            {/if}
            <div class="field full">
              <label>Source</label>
              <input bind:value={editing.source} placeholder="e.g. Dr. Smith (Rheumatology), Idiopathic, Under investigation" />
            </div>
            <div class="field full">
              <label>Personal context</label>
              <textarea bind:value={editing.details} rows="3" placeholder="How this condition manifests for you, relevant history…"></textarea>
            </div>
          </div>
          <div class="form-actions">
            <button class="btn-ghost" onclick={cancelEdit}>Cancel</button>
            <button class="btn-primary" onclick={handleSave} disabled={saving || !editing.name.trim()}>
              {saving ? 'Saving…' : 'Save'}
            </button>
          </div>
          {#if saveError}
            <p class="save-error">{saveError}</p>
          {/if}
        </div>
      {/if}

      {#if diagnoses.length === 0 && !editing}
        <p class="empty">No diagnoses added yet. Add one to give the AI assistant clinical context.</p>
      {/if}

      <div class="diagnosis-list">
        {#each diagnoses as d (d.id)}
          <div class="diagnosis-card">
            <div class="diagnosis-main">
              <div class="diagnosis-name">
                {d.name}
                {#if d.short_name}<span class="short-name">{d.short_name}</span>{/if}
                {#if d.chronic}<span class="badge badge-chronic">chronic</span>{/if}
                {#if d.source?.toLowerCase().includes('idiopathic')}<span class="badge badge-idiopathic">idiopathic</span>{/if}
              </div>
              <div class="diagnosis-meta">
                {#if d.onset_date}<span>Since {d.onset_date}</span>{/if}
                {#if d.source}<span class="source">{d.source}</span>{/if}
              </div>
              {#if d.details}<p class="details">{d.details}</p>{/if}
            </div>
            <div class="diagnosis-actions">
              <button class="btn-ghost btn-sm" onclick={() => startEdit(d)}>Edit</button>
              <button class="btn-ghost btn-sm btn-danger" onclick={() => d.id && handleDelete(d.id)}>Delete</button>
            </div>
          </div>
        {/each}
      </div>
    </section>

    <aside class="right-col">
      <section class="side-card">
        <div class="eyebrow">UPCOMING APPOINTMENTS</div>
        {#each DEMO_APPOINTMENTS as a}
          <div class="side-row">
            <div class="date-tile">
              <div class="date-tile-mo">{fmtMonth(a.date)}</div>
              <div class="date-tile-day">{fmtDay(a.date)}</div>
            </div>
            <div class="side-body">
              <div class="side-title">{a.title}</div>
              <div class="side-sub">{a.location}</div>
            </div>
          </div>
        {/each}
      </section>

      <section class="side-card">
        <div class="eyebrow">PROVIDERS</div>
        {#each DEMO_PROVIDERS as p}
          <div class="side-row">
            <div class="avatar">{p.initials}</div>
            <div class="side-body">
              <div class="side-title">{p.name}</div>
              <div class="side-sub">{p.specialty}</div>
            </div>
          </div>
        {/each}
      </section>
    </aside>
  </div>
</div>

<style>
  .care-team-view { max-width: 1160px; margin: 0 auto; padding: 24px 8px; }
  .care-team-view h1 { font-family: var(--font-heading); font-size: 28px; font-weight: 900; margin-bottom: 20px; }
  .two-col { display: grid; grid-template-columns: 2fr 1fr; gap: 20px; }
  @media (max-width: 900px) { .two-col { grid-template-columns: 1fr; } }

  .col-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px; }
  .eyebrow {
    font-size: 11px; font-weight: 700; letter-spacing: 0.08em;
    color: var(--text-muted); text-transform: uppercase;
  }

  .side-card {
    background: var(--surface); border: 1px solid var(--border);
    border-radius: 12px; padding: 18px 20px; margin-bottom: 16px;
  }
  .side-row { display: flex; align-items: center; gap: 12px; padding: 10px 0; }
  .side-row + .side-row { border-top: 1px solid var(--border); }
  .date-tile {
    background: var(--bg); border: 1px solid var(--border);
    border-radius: 8px; padding: 4px 10px; text-align: center; min-width: 46px;
  }
  .date-tile-mo { font-size: 10px; color: var(--text-muted); letter-spacing: 0.06em; }
  .date-tile-day { font-family: var(--font-heading); font-weight: 700; font-size: 16px; }
  .avatar {
    width: 40px; height: 40px; border-radius: 50%;
    background: var(--primary); color: var(--primary-contrast);
    display: flex; align-items: center; justify-content: center;
    font-weight: 700; font-size: 13px;
  }
  .side-title { font-weight: 600; }
  .side-sub { color: var(--text-muted); font-size: 12px; }

  .form-card { background: var(--surface); border: 1px solid var(--border); border-radius: 8px; padding: 20px; margin-bottom: 24px; }
  .form-card h3 { margin: 0 0 16px; }
  .form-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 12px; }
  .field { display: flex; flex-direction: column; gap: 4px; }
  .field.full { grid-column: 1 / -1; }
  .field.field-inline { flex-direction: row; align-items: center; }
  .field label { font-size: 12px; font-weight: 500; color: var(--text-muted); }
  .field input[type="text"], .field input[type="date"], .field input:not([type="checkbox"]), .field textarea {
    background: var(--surface); border: 1px solid var(--border);
    border-radius: var(--radius); padding: 6px 10px; font-size: 13px; color: var(--color-text);
  }
  .field textarea { resize: vertical; }
  .required { color: var(--color-accent); }
  .form-actions { display: flex; justify-content: flex-end; gap: 8px; margin-top: 16px; }
  .empty { color: var(--text-muted); font-size: 14px; }
  .diagnosis-list { display: flex; flex-direction: column; gap: 10px; }
  .diagnosis-card { background: var(--surface); border: 1px solid var(--border); border-radius: 8px; padding: 14px 16px; display: flex; justify-content: space-between; align-items: flex-start; gap: 16px; }
  .diagnosis-name { font-size: 14px; font-weight: 500; display: flex; align-items: center; gap: 6px; flex-wrap: wrap; margin-bottom: 4px; }
  .short-name { color: var(--text-muted); font-weight: 400; }
  .badge { border-radius: 4px; padding: 1px 6px; font-size: 11px; font-weight: 500; }
  .badge-chronic { background: var(--color-accent); color: white; opacity: 0.85; }
  .badge-idiopathic { background: #cc88ff22; color: #cc88ff; }
  .diagnosis-meta { font-size: 12px; color: var(--text-muted); display: flex; gap: 12px; }
  .details { font-size: 12px; color: var(--text-muted); margin: 6px 0 0; line-height: 1.5; }
  .diagnosis-actions { display: flex; gap: 6px; flex-shrink: 0; }
  .btn-sm { padding: 3px 10px; font-size: 12px; }
  .btn-danger { color: var(--color-error, #e05555); }
  .save-error { color: var(--color-error, #e05555); font-size: 12px; margin-top: 8px; }
</style>
