<script lang="ts">
  import '../app.css';
  import type { View } from '$lib/types';
  import Dashboard from '$lib/views/Dashboard.svelte';
  import LabResults from '$lib/views/LabResults.svelte';
  import LabEntry from '$lib/views/LabEntry.svelte';
  import Trends from '$lib/views/Trends.svelte';
  import SymptomEntry from '$lib/views/SymptomEntry.svelte';
  import SymptomEditor from '$lib/views/SymptomEditor.svelte';
  import Export from '$lib/views/Export.svelte';

  let currentView: View = $state('dashboard');
  let editSessionId: number | null = $state(null);

  function navigate(view: View, sessionId?: number | null) {
    currentView = view;
    editSessionId = sessionId ?? null;
  }

  const navItems: { view: View; label: string; icon: string }[] = [
    { view: 'dashboard', label: 'Dashboard', icon: '~' },
    { view: 'lab-results', label: 'Lab Results', icon: '+' },
    { view: 'trends', label: 'Trends', icon: '^' },
    { view: 'symptoms', label: 'Symptoms', icon: '*' },
    { view: 'symptom-editor', label: 'Manage Symptoms', icon: '#' },
    { view: 'export', label: 'Export', icon: '>' },
  ];
</script>

<div class="app-layout">
  <nav class="sidebar">
    <div class="sidebar-header">
      <h2>Symptom Tracker</h2>
    </div>
    <ul class="nav-list">
      {#each navItems as item}
        <li>
          <button
            class="nav-item"
            class:active={currentView === item.view}
            onclick={() => navigate(item.view)}
          >
            <span class="nav-icon">{item.icon}</span>
            {item.label}
          </button>
        </li>
      {/each}
    </ul>
  </nav>

  <main class="content">
    {#if currentView === 'dashboard'}
      <Dashboard onNavigate={navigate} />
    {:else if currentView === 'lab-results'}
      <LabResults onNavigate={navigate} />
    {:else if currentView === 'lab-entry'}
      <LabEntry sessionId={editSessionId} onNavigate={navigate} />
    {:else if currentView === 'trends'}
      <Trends />
    {:else if currentView === 'symptoms'}
      <SymptomEntry />
    {:else if currentView === 'symptom-editor'}
      <SymptomEditor />
    {:else if currentView === 'export'}
      <Export />
    {/if}
  </main>
</div>

<style>
  .app-layout {
    display: flex;
    height: 100vh;
    overflow: hidden;
  }

  .sidebar {
    width: 200px;
    border-right: 1px solid var(--color-border);
    display: flex;
    flex-direction: column;
    flex-shrink: 0;
  }

  .sidebar-header {
    padding: 16px;
    border-bottom: 1px solid var(--color-border);
  }

  .sidebar-header h2 {
    font-size: 15px;
    font-weight: 600;
  }

  .nav-list {
    list-style: none;
    padding: 8px;
    flex: 1;
  }

  .nav-item {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 8px 12px;
    border: none;
    background: none;
    border-radius: var(--radius);
    font-size: 13px;
    color: var(--color-text-muted);
    text-align: left;
    cursor: pointer;
    transition: background 0.15s, color 0.15s;
  }

  .nav-item:hover {
    background: var(--color-surface-raised);
    color: var(--color-text);
  }

  .nav-item.active {
    background: var(--color-surface-raised);
    color: var(--color-accent);
    font-weight: 500;
  }

  .nav-icon {
    font-family: var(--font-mono);
    font-size: 14px;
    width: 18px;
    text-align: center;
  }

  .content {
    flex: 1;
    overflow-y: auto;
    padding: 24px;
  }
</style>
