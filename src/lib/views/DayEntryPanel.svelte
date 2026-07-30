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
    date; // reset tags only when navigating to a different day
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
  <div class="score-col">
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
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .entry-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }
  .entry-date {
    font-size: 0.85rem;
    color: var(--color-text-muted, #6b7280);
  }
  .close-btn {
    background: none;
    border: none;
    cursor: pointer;
    color: var(--color-text-muted, #6b7280);
    font-size: 1.2rem;
    line-height: 1;
    padding: 0 4px;
  }
  .close-btn:hover { color: var(--color-text, #1a1a1a); }
  .score-col {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .score-btn {
    width: 100%;
    padding: 8px 12px;
    border: 2px solid transparent;
    border-radius: 6px;
    cursor: pointer;
    font-size: 0.85rem;
    font-weight: 500;
    color: white;
    text-shadow: 0 1px 2px rgba(0,0,0,0.4);
    text-align: left;
    transition: transform 0.1s, border-color 0.1s;
  }
  .score-btn:hover { transform: scale(1.02); }
  .score-btn.selected {
    border-color: var(--color-text, #1a1a1a);
    transform: scale(1.02);
  }
</style>
