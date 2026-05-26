<script lang="ts">
  import { onMount } from 'svelte';
  import type { HeatmapConfig } from '$lib/types';
  import { getHeatmapConfig, saveHeatmapConfig } from '$lib/db';

  interface Props {
    onclose: () => void;
    onupdate: (config: HeatmapConfig) => void;
  }
  let { onclose, onupdate }: Props = $props();

  let config = $state<HeatmapConfig>({ steps: 5, labels: [], colors: [] });
  let savedSteps = $state(5);
  let error = $state('');

  onMount(async () => {
    try {
      config = await getHeatmapConfig();
      savedSteps = config.steps;
    } catch (e) {
      error = String(e);
    }
  });

  function addStep() {
    if (config.steps >= 10) return;
    config = {
      steps: config.steps + 1,
      labels: [...config.labels, 'New'],
      colors: [...config.colors, '#888888'],
    };
  }

  function removeStep() {
    if (config.steps <= 2) return;
    config = {
      steps: config.steps - 1,
      labels: config.labels.slice(0, -1),
      colors: config.colors.slice(0, -1),
    };
  }

  function updateLabel(i: number, value: string) {
    const labels = [...config.labels];
    labels[i] = value;
    config = { ...config, labels };
  }

  function updateColor(i: number, value: string) {
    const colors = [...config.colors];
    colors[i] = value;
    config = { ...config, colors };
  }

  async function handleDone() {
    error = '';
    try {
      const oldSteps = config.steps !== savedSteps ? savedSteps : null;
      await saveHeatmapConfig(config, oldSteps);
      onupdate(config);
      onclose();
    } catch (e) {
      error = String(e);
    }
  }

  const stepsChanged = $derived(config.steps !== savedSteps);
  const stepIndices = $derived(Array.from({ length: config.steps }, (_, i) => i));
</script>

<div class="config-aside">
  <div class="config-header">
    <h3>Scale Settings</h3>
    <button class="done-btn primary" onclick={handleDone}>Done</button>
  </div>

  {#if stepsChanged}
    <p class="remap-warning">
      ⚠ Changing the number of steps will remap all existing scores proportionally.
    </p>
  {/if}

  {#if error}
    <p class="error">{error}</p>
  {/if}

  <div class="step-count-row">
    <span>Steps</span>
    <div class="step-controls">
      <button onclick={removeStep} disabled={config.steps <= 2}>−</button>
      <span class="step-number">{config.steps}</span>
      <button onclick={addStep} disabled={config.steps >= 10}>+</button>
    </div>
  </div>

  <div class="step-list">
    {#each stepIndices as i}
      <div class="step-row">
        <input
          type="color"
          value={config.colors[i]}
          oninput={(e) => updateColor(i, (e.target as HTMLInputElement).value)}
          class="color-swatch"
        />
        <input
          type="text"
          value={config.labels[i]}
          oninput={(e) => updateLabel(i, (e.target as HTMLInputElement).value)}
          placeholder="Label"
          class="label-input"
        />
      </div>
    {/each}
  </div>
</div>

<style>
  .config-aside {
    position: fixed;
    top: 0;
    right: 0;
    height: 100%;
    width: 280px;
    background: var(--color-surface, #ffffff);
    border-left: 1px solid var(--color-border, #e5e7eb);
    padding: 20px 16px;
    display: flex;
    flex-direction: column;
    gap: 16px;
    z-index: 200;
    overflow-y: auto;
    box-shadow: -2px 0 8px rgba(0,0,0,0.06);
  }
  .config-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }
  .config-header h3 {
    margin: 0;
    font-size: 1rem;
    color: var(--color-text, #1a1a1a);
  }
  .done-btn {
    padding: 4px 14px;
    font-size: 0.85rem;
  }
  .remap-warning {
    font-size: 0.8rem;
    color: var(--color-warning, #F4B942);
    background: rgba(244,185,66,0.1);
    border-radius: 6px;
    padding: 8px;
    margin: 0;
  }
  .error {
    color: var(--color-danger, #7B0828);
    font-size: 0.8rem;
  }
  .step-count-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    font-size: 0.9rem;
    color: var(--color-text, #1a1a1a);
  }
  .step-controls {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .step-controls button {
    width: 28px;
    height: 28px;
    border-radius: 4px;
    padding: 0;
    font-size: 1rem;
    line-height: 1;
  }
  .step-controls button:disabled {
    opacity: 0.35;
    cursor: default;
  }
  .step-number {
    font-weight: 600;
    min-width: 20px;
    text-align: center;
    color: var(--color-text, #1a1a1a);
  }
  .step-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .step-row {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .color-swatch {
    width: 36px;
    height: 36px;
    padding: 2px;
    border-radius: 4px;
    border: 1px solid var(--color-border, #e5e7eb);
    background: none;
    cursor: pointer;
    flex-shrink: 0;
  }
  .label-input {
    flex: 1;
    padding: 6px 8px;
    border-radius: 4px;
    border: 1px solid var(--color-border, #e5e7eb);
    background: var(--color-surface, #ffffff);
    color: var(--color-text, #1a1a1a);
    font-size: 0.85rem;
  }
</style>
