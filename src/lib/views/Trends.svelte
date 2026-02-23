<script lang="ts">
  import { onMount, tick } from 'svelte';
  import { getAllTestNames, getTrends, getWellnessTrends, getSymptomTrends, getActiveSymptomNames } from '$lib/db';
  import type { TrendPoint, WellnessTrendPoint, SymptomTrendPoint, SymptomNameEntry } from '$lib/types';
  import { Chart, LineController, LineElement, PointElement, LinearScale, CategoryScale, Legend, Tooltip, Filler } from 'chart.js';

  Chart.register(LineController, LineElement, PointElement, LinearScale, CategoryScale, Legend, Tooltip, Filler);

  let { openGlossary }: { openGlossary: (testName?: string) => void } = $props();

  let mode: 'labs' | 'symptoms' = $state('labs');

  // Lab state
  let testNames: string[] = $state([]);
  let selectedTests: string[] = $state([]);

  // Symptom state
  let symptomNames: SymptomNameEntry[] = $state([]);
  let selectedSymptomIds: number[] = $state([]);

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

  let filteredSymptoms = $derived(
    searchQuery
      ? symptomNames.filter(s => s.name.toLowerCase().includes(searchQuery.toLowerCase()))
      : symptomNames
  );

  onMount(async () => {
    try {
      const [names, symptoms] = await Promise.all([getAllTestNames(), getActiveSymptomNames()]);
      testNames = names;
      symptomNames = symptoms;
    } catch (e) {
      console.error('Failed to load names:', e);
    }
  });

  function toggleTest(name: string) {
    if (selectedTests.includes(name)) {
      selectedTests = selectedTests.filter(t => t !== name);
    } else {
      selectedTests = [...selectedTests, name];
    }
  }

  function toggleSymptom(id: number) {
    if (selectedSymptomIds.includes(id)) {
      selectedSymptomIds = selectedSymptomIds.filter(i => i !== id);
    } else {
      selectedSymptomIds = [...selectedSymptomIds, id];
    }
  }

  const COLORS = ['#2563eb', '#dc2626', '#16a34a', '#d97706', '#7c3aed', '#0891b2', '#be123c', '#4f46e5'];

  async function loadLabChart() {
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
        data: { labels: sortedDates, datasets },
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
            tooltip: { callbacks: { title: (items) => items[0]?.label || '' } },
          },
        },
      });
    } catch (e) {
      console.error('Failed to load trends:', e);
    }
    loading = false;
  }

  async function loadSymptomChart() {
    loading = true;
    try {
      const datasets: any[] = [];
      let allDates: Set<string> = new Set();

      // Always fetch wellness data
      const wellness: WellnessTrendPoint[] = await getWellnessTrends(days);
      for (const p of wellness) allDates.add(p.date);

      datasets.push({
        label: 'Wellness Score',
        data: wellness.map(p => ({ x: p.date, y: p.wellness_score })),
        borderColor: '#2563eb',
        backgroundColor: '#2563eb20',
        tension: 0.3,
        pointRadius: 4,
        pointHoverRadius: 6,
        yAxisID: 'y',
      });

      // Individual symptoms
      for (let i = 0; i < selectedSymptomIds.length; i++) {
        const id = selectedSymptomIds[i];
        const name = symptomNames.find(s => s.id === id)?.name || `Symptom ${id}`;
        const points: SymptomTrendPoint[] = await getSymptomTrends(id, days);
        for (const p of points) allDates.add(p.date);

        datasets.push({
          label: name,
          data: points.map(p => ({ x: p.date, y: p.present ? 1 : 0 })),
          borderColor: COLORS[(i + 1) % COLORS.length],
          backgroundColor: COLORS[(i + 1) % COLORS.length] + '20',
          stepped: 'middle',
          pointRadius: 3,
          pointHoverRadius: 5,
          yAxisID: 'y1',
        });
      }

      const sortedDates = [...allDates].sort();
      await tick();

      if (chart) chart.destroy();
      if (!chartCanvas) return;

      chart = new Chart(chartCanvas, {
        type: 'line',
        data: { labels: sortedDates, datasets },
        options: {
          responsive: true,
          maintainAspectRatio: false,
          interaction: { mode: 'index', intersect: false },
          scales: {
            x: { grid: { color: '#e5e7eb' } },
            y: {
              type: 'linear',
              position: 'left',
              min: 1,
              max: 5,
              title: { display: true, text: 'Wellness (1-5)' },
              grid: { color: '#e5e7eb' },
              ticks: { stepSize: 1 },
            },
            y1: {
              type: 'linear',
              position: 'right',
              min: -0.1,
              max: 1.1,
              title: { display: true, text: 'Present' },
              grid: { drawOnChartArea: false },
              ticks: {
                stepSize: 1,
                callback: (val: string | number) => val === 1 ? 'Yes' : val === 0 ? 'No' : '',
              },
            },
          },
          plugins: {
            legend: { position: 'top' },
            tooltip: { callbacks: { title: (items) => items[0]?.label || '' } },
          },
        },
      });
    } catch (e) {
      console.error('Failed to load symptom trends:', e);
    }
    loading = false;
  }

  $effect(() => {
    if (mode === 'labs') {
      if (selectedTests.length >= 0) loadLabChart();
    } else {
      // Trigger on selectedSymptomIds or days change
      void selectedSymptomIds;
      void days;
      loadSymptomChart();
    }
  });

  function switchMode(m: 'labs' | 'symptoms') {
    mode = m;
    searchQuery = '';
    if (chart) { chart.destroy(); chart = null; }
  }
</script>

<div class="trends">
  <h1>Trends</h1>

  <div class="mode-toggle">
    <button class="mode-btn" class:active={mode === 'labs'} onclick={() => switchMode('labs')}>Lab Results</button>
    <button class="mode-btn" class:active={mode === 'symptoms'} onclick={() => switchMode('symptoms')}>Symptoms</button>
  </div>

  <div class="controls">
    {#if mode === 'labs'}
      <div class="test-selector">
        <label for="test-search">Select Tests</label>
        <input id="test-search" type="text" bind:value={searchQuery} placeholder="Search tests..." />
        <div class="test-list">
          {#if testNames.length === 0}
            <p class="muted">No lab results recorded yet.</p>
          {:else}
            {#each filteredTests as name}
              <div class="test-option-row">
                <label class="test-option">
                  <input
                    type="checkbox"
                    checked={selectedTests.includes(name)}
                    onchange={() => toggleTest(name)}
                  />
                  {name}
                </label>
                <button class="info-btn" onclick={() => openGlossary(name)} title="View in glossary">i</button>
              </div>
            {/each}
          {/if}
        </div>
      </div>
    {:else}
      <div class="test-selector">
        <label for="symptom-search">Select Symptoms</label>
        <input id="symptom-search" type="text" bind:value={searchQuery} placeholder="Search symptoms..." />
        <div class="test-list">
          <div class="test-option-row wellness-row">
            <label class="test-option">
              <input type="checkbox" checked disabled />
              Wellness Score
            </label>
            <span class="muted" style="font-size: 11px">always shown</span>
          </div>
          {#if symptomNames.length === 0}
            <p class="muted">No symptoms configured yet.</p>
          {:else}
            {#each filteredSymptoms as s}
              <div class="test-option-row">
                <label class="test-option">
                  <input
                    type="checkbox"
                    checked={selectedSymptomIds.includes(s.id)}
                    onchange={() => toggleSymptom(s.id)}
                  />
                  {s.name}
                </label>
              </div>
            {/each}
          {/if}
        </div>
      </div>
    {/if}

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
    {:else if mode === 'labs' && selectedTests.length === 0}
      <p class="muted">Select one or more tests to view trends.</p>
    {/if}
    <canvas bind:this={chartCanvas}></canvas>
  </div>
</div>

<style>
  .trends { max-width: 960px; }

  h1 { margin-bottom: 20px; }

  .mode-toggle {
    display: flex;
    gap: 0;
    margin-bottom: 16px;
  }

  .mode-btn {
    padding: 6px 16px;
    font-size: 13px;
    font-weight: 500;
    border: 1px solid var(--color-border);
    background: var(--color-surface);
    color: var(--color-text-muted);
    cursor: pointer;
    transition: all 0.15s;
  }

  .mode-btn:first-child {
    border-radius: var(--radius) 0 0 var(--radius);
  }

  .mode-btn:last-child {
    border-radius: 0 var(--radius) var(--radius) 0;
    margin-left: -1px;
  }

  .mode-btn.active {
    background: var(--color-accent);
    color: white;
    border-color: var(--color-accent);
    z-index: 1;
    position: relative;
  }

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

  .test-option-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 1px 4px;
    border-radius: 3px;
  }

  .test-option-row:hover {
    background: var(--color-surface-raised);
  }

  .wellness-row {
    border-bottom: 1px solid var(--color-border);
    padding-bottom: 4px;
    margin-bottom: 4px;
  }

  .test-option:hover {
    background: none;
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
