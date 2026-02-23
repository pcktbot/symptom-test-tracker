<script lang="ts">
  import { onMount, tick } from 'svelte';
  import { getMergedGlossary } from '$lib/glossary';
  import type { GlossaryEntry } from '$lib/glossary';

  let { activeTest = null, onClose }: { activeTest: string | null; onClose: () => void } = $props();

  let grouped: Record<string, GlossaryEntry[]> = $state({});
  let activeEl: HTMLElement | undefined = $state();

  onMount(async () => {
    grouped = await getMergedGlossary();

    function handleKeydown(e: KeyboardEvent) {
      if (e.key === 'Escape') onClose();
    }
    window.addEventListener('keydown', handleKeydown);
    return () => window.removeEventListener('keydown', handleKeydown);
  });

  $effect(() => {
    if (activeTest && activeEl) {
      tick().then(() => {
        const el = document.getElementById(`glossary-${CSS.escape(activeTest!)}`);
        if (el) {
          el.scrollIntoView({ behavior: 'smooth', block: 'center' });
          el.classList.add('highlight');
          setTimeout(() => el.classList.remove('highlight'), 1500);
        }
      });
    }
  });
</script>

<aside class="glossary" bind:this={activeEl}>
  <div class="glossary-header">
    <h2>Glossary</h2>
    <button class="close-btn" onclick={onClose} aria-label="Close glossary">&times;</button>
  </div>

  <div class="glossary-body">
    {#each Object.entries(grouped) as [panel, entries]}
      <div class="panel-section">
        <h3>{panel}</h3>
        {#each entries as entry}
          <div class="glossary-entry" id="glossary-{entry.name}">
            <div class="entry-name">{entry.name}</div>
            <p class="entry-desc">{entry.description}</p>
            <p class="entry-clinical"><span class="clinical-label">Clinical:</span> {entry.clinical}</p>
          </div>
        {/each}
      </div>
    {/each}
  </div>
</aside>

<style>
  .glossary {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--color-surface);
    border-left: 1px solid var(--color-border);
    overflow: hidden;
  }

  .glossary-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 16px 16px 12px;
    border-bottom: 1px solid var(--color-border);
    flex-shrink: 0;
  }

  .glossary-header h2 {
    font-size: 15px;
  }

  .close-btn {
    width: 28px;
    height: 28px;
    display: flex;
    align-items: center;
    justify-content: center;
    border: none;
    background: none;
    font-size: 20px;
    color: var(--color-text-muted);
    cursor: pointer;
    border-radius: var(--radius);
    padding: 0;
    line-height: 1;
  }

  .close-btn:hover {
    background: var(--color-surface-raised);
    color: var(--color-text);
  }

  .glossary-body {
    flex: 1;
    overflow-y: auto;
    padding: 12px 16px;
  }

  .panel-section {
    margin-bottom: 20px;
  }

  .panel-section h3 {
    font-size: 12px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--color-text-muted);
    padding-bottom: 4px;
    border-bottom: 1px solid var(--color-border);
    margin-bottom: 8px;
  }

  .glossary-entry {
    padding: 8px;
    border-radius: var(--radius);
    margin-bottom: 4px;
    transition: background 0.3s;
  }

  :global(.glossary-entry.highlight) {
    background: #fffbeb;
  }

  .entry-name {
    font-weight: 600;
    font-size: 13px;
    margin-bottom: 2px;
  }

  .entry-desc {
    font-size: 12px;
    color: var(--color-text);
    line-height: 1.5;
    margin-bottom: 4px;
  }

  .entry-clinical {
    font-size: 12px;
    color: var(--color-text-muted);
    line-height: 1.5;
  }

  .clinical-label {
    font-weight: 600;
    color: var(--color-text);
  }
</style>
