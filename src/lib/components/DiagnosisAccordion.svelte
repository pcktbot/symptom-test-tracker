<script lang="ts">
  import { getDiagnoses } from '$lib/db';
  import type { Diagnosis, View } from '$lib/types';

  let { onNavigate }: { onNavigate: (view: View) => void } = $props();

  let diagnoses = $state<Diagnosis[]>([]);
  let open = $state(false);

  $effect(() => {
    getDiagnoses().then(d => diagnoses = d);
  });
</script>

{#if diagnoses.length > 0}
  <div class="accordion">
    <button class="accordion-header" onclick={() => open = !open} aria-expanded={open}>
      <div class="accordion-left">
        <span class="accordion-label">Diagnoses</span>
        <span class="count-badge">{diagnoses.length}</span>
      </div>
      <span class="chevron" class:open>{open ? '▲' : '▼'}</span>
    </button>

    {#if open}
      <div class="accordion-body">
        {#each diagnoses as d (d.id)}
          <div class="diagnosis-row">
            <div class="diagnosis-info">
              <span class="diag-name">{d.name}</span>
              {#if d.short_name}<span class="diag-short">{d.short_name}</span>{/if}
              {#if d.chronic}<span class="badge-chronic">chronic</span>{/if}
              {#if d.source?.toLowerCase().includes('idiopathic')}<span class="badge-idiopathic">idiopathic</span>{/if}
            </div>
            {#if d.onset_date}<span class="diag-date">since {d.onset_date}</span>{/if}
          </div>
        {/each}
        <div class="accordion-footer">
          <button class="link-btn" onclick={() => onNavigate('care-team')}>Edit in Profile →</button>
        </div>
      </div>
    {/if}
  </div>
{/if}

<style>
  .accordion { border: 1px solid var(--color-border); border-radius: 6px; margin-bottom: 20px; overflow: hidden; }
  .accordion-header { display: flex; align-items: center; justify-content: space-between; width: 100%; padding: 8px 12px; background: var(--color-surface-raised); border: none; cursor: pointer; font-size: 13px; }
  .accordion-header:hover { background: var(--color-surface); }
  .accordion-left { display: flex; align-items: center; gap: 8px; }
  .accordion-label { font-size: 11px; font-weight: 500; text-transform: uppercase; letter-spacing: 0.05em; color: var(--color-text-muted); }
  .count-badge { background: var(--color-surface); border: 1px solid var(--color-border); border-radius: 10px; padding: 1px 7px; font-size: 10px; color: var(--color-text-muted); }
  .chevron { font-size: 9px; color: var(--color-text-muted); }
  .accordion-body { border-top: 1px solid var(--color-border); padding: 10px 12px; display: flex; flex-direction: column; gap: 6px; background: var(--color-surface); }
  .diagnosis-row { display: flex; align-items: center; justify-content: space-between; }
  .diagnosis-info { display: flex; align-items: center; gap: 6px; flex-wrap: wrap; }
  .diag-name { font-size: 13px; font-weight: 500; }
  .diag-short { font-size: 12px; color: var(--color-text-muted); }
  .badge-chronic { background: var(--color-accent); color: white; border-radius: 3px; padding: 1px 5px; font-size: 10px; opacity: 0.85; }
  .badge-idiopathic { background: #cc88ff22; color: #cc88ff; border-radius: 3px; padding: 1px 5px; font-size: 10px; }
  .diag-date { font-size: 11px; color: var(--color-text-muted); flex-shrink: 0; }
  .accordion-footer { border-top: 1px solid var(--color-border); margin-top: 4px; padding-top: 8px; }
  .link-btn { background: none; border: none; color: var(--color-accent); font-size: 12px; cursor: pointer; padding: 0; }
  .link-btn:hover { text-decoration: underline; }
</style>
