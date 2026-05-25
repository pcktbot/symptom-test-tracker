<script lang="ts">
  import TagInput from '$lib/TagInput.svelte';
  import { scoreToColor, scoreToLabel } from '$lib/heatmap';
  import type { DailyRating, HeatmapConfig } from '$lib/types';

  interface Props {
    date: string;
    existing: DailyRating | null;
    config: HeatmapConfig;
    allTags: string[];
    onsave: (score: number, tags: string[]) => void;
    onclose: () => void;
  }
  let { date, existing, config, allTags, onsave, onclose }: Props = $props();

  let currentTags = $state<string[]>([]);

  $effect(() => {
    currentTags = existing?.tags ?? [];
  });

  function selectScore(score: number) {
    onsave(score, currentTags);
  }

  function updateTags(newTags: string[]) {
    currentTags = newTags;
    if (existing) {
      onsave(existing.wellness_score, newTags);
    }
  }

  const steps = $derived(Array.from({ length: config.steps }, (_, i) => i + 1));

  function formatDate(d: string): string {
    const [y, m, day] = d.split('-');
    return new Date(Number(y), Number(m) - 1, Number(day)).toLocaleDateString('en-US', {
      weekday: 'short', month: 'short', day: 'numeric', year: 'numeric',
    });
  }
</script>

<div class="entry-panel">
  <div class="entry-header">
    <span class="entry-date">{formatDate(date)}</span>
    <button class="close-btn" onclick={onclose} title="Close">×</button>
  </div>
  <div class="score-row">
    {#each steps as step}
      <button
        class="score-btn"
        class:selected={existing?.wellness_score === step}
        style="background-color: {scoreToColor(step, config)}"
        onclick={() => selectScore(step)}
        title={scoreToLabel(step, config)}
      >
        {scoreToLabel(step, config)}
      </button>
    {/each}
  </div>
  <TagInput tags={currentTags} suggestions={allTags} onchange={updateTags} />
</div>

<style>
  .entry-panel {
    margin-top: 16px;
    padding: 16px;
    border: 1px solid var(--border, #444);
    border-radius: 8px;
    background: var(--surface, #1e1e1e);
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  .entry-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }
  .entry-date {
    font-size: 0.9rem;
    color: var(--text-muted, #aaa);
  }
  .close-btn {
    background: none;
    border: none;
    cursor: pointer;
    color: var(--text-muted, #888);
    font-size: 1.2rem;
    line-height: 1;
    padding: 0 4px;
  }
  .close-btn:hover { color: var(--text, #e0e0e0); }
  .score-row {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
  }
  .score-btn {
    flex: 1;
    min-width: 60px;
    padding: 8px 4px;
    border: 2px solid transparent;
    border-radius: 6px;
    cursor: pointer;
    font-size: 0.8rem;
    font-weight: 500;
    color: white;
    text-shadow: 0 1px 2px rgba(0,0,0,0.5);
    transition: transform 0.1s, border-color 0.1s;
  }
  .score-btn:hover { transform: scale(1.04); }
  .score-btn.selected {
    border-color: white;
    transform: scale(1.06);
  }
</style>
