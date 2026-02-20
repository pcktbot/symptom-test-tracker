<script lang="ts">
  import { onMount, tick } from 'svelte';
  import { getAllTestNames, getTrends } from '$lib/db';
  import type { TrendPoint } from '$lib/types';
  import { Chart, LineController, LineElement, PointElement, LinearScale, CategoryScale, Legend, Tooltip, Filler } from 'chart.js';

  Chart.register(LineController, LineElement, PointElement, LinearScale, CategoryScale, Legend, Tooltip, Filler);

  let testNames: string[] = $state([]);
  let selectedTests: string[] = $state([]);
  let days = $state(365);
  let searchQuery = $state('');
  let loading = $state(false);
  let chartCanvas: HTMLCanvasElement | undefined = $state();
  let chart: Chart | null = null;

  let filteredTests = $derived(
    searchQuery
      ? testNames.filter(t => t.toLowerCase().includes(searchQuery.toLowerCase()))
      : testNames
  );

  onMount(async () => {
    try {
      testNames = await getAllTestNames();
    } catch (e) {
      console.error('Failed to load test names:', e);
    }
  });

  function toggleTest(name: string) {
    if (selectedTests.includes(name)) {
      selectedTests = selectedTests.filter(t => t !== name);
    } else {
      selectedTests = [...selectedTests, name];
    }
  }

  const COLORS = ['#2563eb', '#dc2626', '#16a34a', '#d97706', '#7c3aed', '#0891b2', '#be123c', '#4f46e5'];

  async function loadChart() {
    if (selectedTests.length === 0) {
      if (chart) { chart.destroy(); chart = null; }
      return;
    }
    loading = true;

    try {
      const datasets = [];
      let allDates: Set<string> = new Set();

      for (let i = 0; i < selectedTests.length; i++) {
        const test = selectedTests[i];
        const points: TrendPoint[] = await getTrends(test, days);
        for (const p of points) allDates.add(p.test_date);

        datasets.push({
          label: test,
          data: points.filter(p => p.value != null).map(p => ({ x: p.test_date, y: p.value! })),
          borderColor: COLORS[i % COLORS.length],
          backgroundColor: COLORS[i % COLORS.length] + '20',
          tension: 0.3,
          pointRadius: 4,
          pointHoverRadius: 6,
        });
      }

      const sortedDates = [...allDates].sort();
      await tick();

      if (chart) chart.destroy();
      if (!chartCanvas) return;

      chart = new Chart(chartCanvas, {
        type: 'line',
        data: {
          labels: sortedDates,
          datasets,
        },
        options: {
          responsive: true,
          maintainAspectRatio: false,
          interaction: { mode: 'index', intersect: false },
          scales: {
            x: { grid: { color: '#e5e7eb' } },
            y: { grid: { color: '#e5e7eb' }, beginAtZero: false },
          },
          plugins: {
            legend: { position: 'top' },
            tooltip: {
              callbacks: {
                title: (items) => items[0]?.label || '',
              },
            },
          },
        },
      });
    } catch (e) {
      console.error('Failed to load trends:', e);
    }
    loading = false;
  }

  $effect(() => {
    // Re-render chart whenever selectedTests or days change
    if (selectedTests.length >= 0) {
      loadChart();
    }
  });
</script>

<div class="trends">
  <h1>Trends</h1>

  <div class="controls">
    <div class="test-selector">
      <label for="test-search">Select Tests</label>
      <input id="test-search" type="text" bind:value={searchQuery} placeholder="Search tests..." />
      <div class="test-list">
        {#if testNames.length === 0}
          <p class="muted">No lab results recorded yet.</p>
        {:else}
          {#each filteredTests as name}
            <label class="test-option">
              <input
                type="checkbox"
                checked={selectedTests.includes(name)}
                onchange={() => toggleTest(name)}
              />
              {name}
            </label>
          {/each}
        {/if}
      </div>
    </div>

    <div class="field">
      <label for="days-range">Time Range</label>
      <select id="days-range" bind:value={days}>
        <option value={30}>30 days</option>
        <option value={90}>90 days</option>
        <option value={180}>6 months</option>
        <option value={365}>1 year</option>
        <option value={730}>2 years</option>
        <option value={9999}>All time</option>
      </select>
    </div>
  </div>

  <div class="chart-container">
    {#if loading}
      <p class="muted">Loading...</p>
    {:else if selectedTests.length === 0}
      <p class="muted">Select one or more tests to view trends.</p>
    {/if}
    <canvas bind:this={chartCanvas}></canvas>
  </div>
</div>

<style>
  .trends { max-width: 960px; }

  h1 { margin-bottom: 20px; }

  .controls {
    display: flex;
    gap: 16px;
    margin-bottom: 20px;
  }

  .test-selector {
    flex: 1;
  }

  .test-selector label:first-child {
    display: block;
    margin-bottom: 4px;
  }

  .test-selector input[type="text"] {
    width: 100%;
    margin-bottom: 6px;
  }

  .test-list {
    max-height: 200px;
    overflow-y: auto;
    border: 1px solid var(--color-border);
    border-radius: var(--radius);
    padding: 6px;
  }

  .test-option {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 3px 4px;
    font-size: 13px;
    color: var(--color-text);
    cursor: pointer;
    border-radius: 3px;
  }

  .test-option:hover {
    background: var(--color-surface-raised);
  }

  .test-option input[type="checkbox"] {
    margin: 0;
  }

  .field {
    display: flex;
    flex-direction: column;
  }

  .field select {
    min-width: 120px;
  }

  .chart-container {
    height: 400px;
    border: 1px solid var(--color-border);
    border-radius: var(--radius);
    padding: 16px;
    display: flex;
    align-items: center;
    justify-content: center;
    position: relative;
  }

  .chart-container canvas {
    width: 100% !important;
    height: 100% !important;
  }

  .muted { color: var(--color-text-muted); }
</style>
