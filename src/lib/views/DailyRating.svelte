<script lang="ts">
  import { onMount } from 'svelte';
  import HeatmapGrid from '$lib/views/HeatmapGrid.svelte';
  import DayEntryPanel from '$lib/views/DayEntryPanel.svelte';
  import HeatmapConfig from '$lib/views/HeatmapConfig.svelte';
  import type { DailyRating, HeatmapConfig as HeatmapConfigType } from '$lib/types';
  import { getDailyRatings, saveDailyRating, getAllTags, getHeatmapConfig } from '$lib/db';
  import { DEFAULT_CONFIG } from '$lib/heatmap';

  let year = $state(new Date().getFullYear());
  let ratings = $state<Map<string, DailyRating>>(new Map());
  let selectedDate = $state<string | null>(null);
  let config = $state<HeatmapConfigType>(DEFAULT_CONFIG);
  let allTags = $state<string[]>([]);
  let configOpen = $state(false);
  let error = $state('');

  const currentYear = new Date().getFullYear();

  async function loadYear(y: number) {
    try {
      const list = await getDailyRatings(y);
      ratings = new Map(list.map(r => [r.log_date, r]));
    } catch (e) {
      error = String(e);
    }
  }

  onMount(async () => {
    try {
      [config, allTags] = await Promise.all([getHeatmapConfig(), getAllTags()]);
    } catch (e) {
      error = String(e);
    }
  });

  $effect(() => {
    loadYear(year);
  });

  async function handleSave(score: number, tags: string[]) {
    if (!selectedDate) return;
    try {
      await saveDailyRating(selectedDate, score, tags);
      const updated: DailyRating = { log_date: selectedDate, wellness_score: score, tags };
      const next = new Map(ratings);
      next.set(selectedDate, updated);
      ratings = next;
      const newTags = tags.filter(t => !allTags.includes(t));
      if (newTags.length) allTags = [...allTags, ...newTags].sort();
    } catch (e) {
      error = String(e);
    }
  }

  function handleSelect(date: string) {
    selectedDate = selectedDate === date ? null : date;
  }

  function handleConfigUpdate(newConfig: HeatmapConfigType) {
    config = newConfig;
    loadYear(year);
  }

  const selectedRating = $derived(selectedDate ? (ratings.get(selectedDate) ?? null) : null);
</script>

<svelte:window
  onkeydown={(e) => {
    if (e.key === 'Escape') selectedDate = null;
  }}
/>

<div class="daily-rating-view">
  <div class="view-header">
    <div class="year-nav">
      <button onclick={() => { year--; selectedDate = null; }} title="Previous year">‹</button>
      <span class="year-label">{year}</span>
      <button
        onclick={() => { year++; selectedDate = null; }}
        disabled={year >= currentYear}
        title="Next year"
      >›</button>
    </div>
    <button
      class="config-toggle"
      onclick={() => (configOpen = !configOpen)}
      title="Configure scale"
      class:active={configOpen}
    >⚙</button>
  </div>

  {#if error}
    <p class="error-msg">{error}</p>
  {/if}

  <div class="layout">
    <div class="left-col">
      <HeatmapGrid {ratings} {config} {selectedDate} {year} onselect={handleSelect} />
    </div>
    <div class="right-col">
      {#if selectedDate}
        <DayEntryPanel
          date={selectedDate}
          existing={selectedRating}
          {config}
          {allTags}
          onsave={handleSave}
          onclose={() => (selectedDate = null)}
        />
      {:else}
        <p class="no-selection">Select a day to log a rating.</p>
      {/if}
    </div>
  </div>

  {#if configOpen}
    <HeatmapConfig onclose={() => (configOpen = false)} onupdate={handleConfigUpdate} />
  {/if}
</div>

<style>
  .daily-rating-view {
    padding: 20px;
    max-width: 1100px;
    margin: 0 auto;
    height: 100%;
    display: flex;
    flex-direction: column;
  }
  .view-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 16px;
  }
  .year-nav {
    display: flex;
    align-items: center;
    gap: 12px;
  }
  .year-nav button {
    background: none;
    border: 1px solid var(--border);
    border-radius: 4px;
    color: var(--text);
    width: 28px;
    height: 28px;
    cursor: pointer;
    font-size: 1.2rem;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 0;
    line-height: 1;
  }
  .year-nav button:hover {
    background: var(--surface);
    border-color: var(--border);
  }
  .year-nav button:disabled {
    opacity: 0.35;
    cursor: default;
  }
  .year-label {
    font-size: 1.1rem;
    font-weight: 600;
    min-width: 52px;
    text-align: center;
    color: var(--text);
  }
  .config-toggle {
    background: none;
    border: 1px solid var(--border);
    border-radius: 6px;
    color: var(--text-muted);
    width: 32px;
    height: 32px;
    cursor: pointer;
    font-size: 1rem;
    padding: 0;
  }
  .config-toggle:hover, .config-toggle.active {
    color: var(--text);
    border-color: var(--border);
    background: var(--surface);
  }
  .error-msg {
    color: var(--color-danger, #7B0828);
    font-size: 0.85rem;
    margin: 0 0 12px;
  }
  .layout {
    display: flex;
    gap: 24px;
    flex: 1;
    overflow: hidden;
    min-height: 0;
  }
  .left-col {
    flex: 0 0 auto;
    overflow-y: auto;
  }
  .right-col {
    flex: 1;
    min-width: 180px;
    overflow-y: auto;
    padding-top: 36px; /* align with grid body (below month headers) */
  }
  .no-selection {
    font-size: 0.85rem;
    color: var(--text-muted);
  }
</style>
