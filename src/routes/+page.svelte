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
  import Glossary from '$lib/views/Glossary.svelte';
  import LabManage from '$lib/views/LabManage.svelte';
  import Welcome from '$lib/views/Welcome.svelte';
  import Settings from '$lib/views/Settings.svelte';

  const welcomeSeen = typeof localStorage !== 'undefined' && localStorage.getItem('welcome_seen') === 'true';
  let currentView: View = $state(welcomeSeen ? 'dashboard' : 'welcome');
  let editSessionId: number | null = $state(null);
  let settingsOpen = $state(false);
  let glossaryOpen = $state(false);
  let glossaryTest: string | null = $state(null);

  // Track body area width to decide inline vs overlay
  // We measure body-area (not content) to avoid resize loops when glossary toggles
  let bodyArea: HTMLElement | undefined = $state();
  let bodyWidth = $state(0);

  const GLOSSARY_WIDTH = 340;
  const CONTENT_MAX = 960; // largest max-width among views

  // Glossary fits inline when the body area is wide enough for both
  let glossaryInline = $derived(bodyWidth >= CONTENT_MAX + GLOSSARY_WIDTH + 48);

  function navigate(view: View, sessionId?: number | null) {
    currentView = view;
    editSessionId = sessionId ?? null;
  }

  function dismissWelcome() {
    localStorage.setItem('welcome_seen', 'true');
    currentView = 'dashboard';
  }

  function openGlossary(testName?: string) {
    glossaryTest = testName ?? null;
    glossaryOpen = true;
  }

  function closeGlossary() {
    glossaryOpen = false;
    glossaryTest = null;
  }

  // Observe body area resize (stable — not affected by glossary toggling)
  $effect(() => {
    if (!bodyArea) return;
    const ro = new ResizeObserver(entries => {
      for (const entry of entries) {
        bodyWidth = entry.contentRect.width;
      }
    });
    ro.observe(bodyArea);
    return () => ro.disconnect();
  });

  type NavGroup = { label: string; items: { view: View; label: string }[] };

  const navGroups: NavGroup[] = [
    {
      label: 'Overview',
      items: [
        { view: 'dashboard', label: 'Dashboard' },
      ],
    },
    {
      label: 'Labs',
      items: [
        { view: 'lab-results', label: 'Results' },
        { view: 'trends', label: 'Trends' },
        { view: 'lab-manage', label: 'Manage' },
      ],
    },
    {
      label: 'Symptoms',
      items: [
        { view: 'symptoms', label: 'Log' },
        { view: 'symptom-editor', label: 'Manage' },
      ],
    },
    {
      label: 'Data',
      items: [
        { view: 'export', label: 'Export' },
      ],
    },
  ];
</script>

<div class="app-layout">
  <header class="toolbar">
    <div class="toolbar-left">
    <div class="toolbar-title">Symptom Tracker</div>
    <button class="about-btn" onclick={() => navigate('welcome')} title="About this app">?</button>
    <button class="about-btn" onclick={() => settingsOpen = true} title="Settings">
      <svg width="12" height="12" viewBox="0 0 16 16" fill="currentColor">
        <path d="M8 4.754a3.246 3.246 0 1 0 0 6.492 3.246 3.246 0 0 0 0-6.492zM5.754 8a2.246 2.246 0 1 1 4.492 0 2.246 2.246 0 0 1-4.492 0z"/>
        <path d="M9.796 1.343c-.527-1.79-3.065-1.79-3.592 0l-.094.319a.873.873 0 0 1-1.255.52l-.292-.16c-1.64-.892-3.433.902-2.54 2.541l.159.292a.873.873 0 0 1-.52 1.255l-.319.094c-1.79.527-1.79 3.065 0 3.592l.319.094a.873.873 0 0 1 .52 1.255l-.16.292c-.892 1.64.902 3.434 2.541 2.54l.292-.159a.873.873 0 0 1 1.255.52l.094.319c.527 1.79 3.065 1.79 3.592 0l.094-.319a.873.873 0 0 1 1.255-.52l.292.16c1.64.893 3.434-.902 2.54-2.541l-.159-.292a.873.873 0 0 1 .52-1.255l.319-.094c1.79-.527 1.79-3.065 0-3.592l-.319-.094a.873.873 0 0 1-.52-1.255l.16-.292c.893-1.64-.902-3.433-2.541-2.54l-.292.159a.873.873 0 0 1-1.255-.52l-.094-.319zm-2.633.283c.246-.835 1.428-.835 1.674 0l.094.319a1.873 1.873 0 0 0 2.693 1.115l.291-.16c.764-.415 1.6.42 1.184 1.185l-.159.292a1.873 1.873 0 0 0 1.116 2.692l.318.094c.835.246.835 1.428 0 1.674l-.319.094a1.873 1.873 0 0 0-1.115 2.693l.16.291c.415.764-.421 1.6-1.185 1.184l-.291-.159a1.873 1.873 0 0 0-2.693 1.116l-.094.318c-.246.835-1.428.835-1.674 0l-.094-.319a1.873 1.873 0 0 0-2.692-1.115l-.292.16c-.764.415-1.6-.421-1.184-1.185l.159-.291A1.873 1.873 0 0 0 1.945 8.93l-.319-.094c-.835-.246-.835-1.428 0-1.674l.319-.094A1.873 1.873 0 0 0 3.06 4.377l-.16-.292c-.415-.764.421-1.6 1.185-1.184l.292.159a1.873 1.873 0 0 0 2.692-1.116l.094-.318z"/>
      </svg>
    </button>
  </div>
    <nav class="toolbar-nav">
      {#each navGroups as group}
        <div class="btn-group">
          <span class="group-label">{group.label}</span>
          <div class="group-buttons">
            {#each group.items as item}
              <button
                class="toolbar-btn"
                class:active={currentView === item.view}
                onclick={() => navigate(item.view)}
              >
                {item.label}
              </button>
            {/each}
          </div>
        </div>
      {/each}
    </nav>
  </header>

  <div class="body-area" bind:this={bodyArea}>
    <main class="content">
      {#if currentView === 'welcome'}
        <Welcome onDismiss={dismissWelcome} />
      {:else if currentView === 'dashboard'}
        <Dashboard onNavigate={navigate} {openGlossary} />
      {:else if currentView === 'lab-results'}
        <LabResults onNavigate={navigate} {openGlossary} />
      {:else if currentView === 'lab-entry'}
        <LabEntry sessionId={editSessionId} onNavigate={navigate} {openGlossary} />
      {:else if currentView === 'trends'}
        <Trends {openGlossary} />
      {:else if currentView === 'symptoms'}
        <SymptomEntry />
      {:else if currentView === 'symptom-editor'}
        <SymptomEditor />
      {:else if currentView === 'lab-manage'}
        <LabManage />
      {:else if currentView === 'export'}
        <Export />
      {/if}
    </main>

    {#if glossaryOpen && glossaryInline}
      <div class="glossary-inline" style="width: {GLOSSARY_WIDTH}px">
        <Glossary activeTest={glossaryTest} onClose={closeGlossary} />
      </div>
    {/if}
  </div>

  {#if glossaryOpen && !glossaryInline}
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="glossary-overlay-backdrop" onclick={closeGlossary} onkeydown={() => {}}>
    </div>
    <div class="glossary-overlay" style="width: {GLOSSARY_WIDTH}px">
      <Glossary activeTest={glossaryTest} onClose={closeGlossary} />
    </div>
  {/if}

  {#if settingsOpen}
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="settings-backdrop" onclick={() => settingsOpen = false} onkeydown={() => {}}></div>
    <div class="settings-modal">
      <Settings onClose={() => settingsOpen = false} />
    </div>
  {/if}
</div>

<style>
  .app-layout {
    display: flex;
    flex-direction: column;
    height: 100vh;
    overflow: hidden;
    position: relative;
  }

  .toolbar {
    display: flex;
    align-items: center;
    gap: 24px;
    padding: 16px 24px;
    border-bottom: 1px solid var(--color-border);
    flex-shrink: 0;
  }

  .toolbar-left {
    display: flex;
    align-items: center;
    gap: 8px;
    padding-right: 8px;
    border-right: 1px solid var(--color-border);
  }

  .toolbar-title {
    font-size: 15px;
    font-weight: 600;
    white-space: nowrap;
  }

  .about-btn {
    width: 22px;
    height: 22px;
    display: flex;
    align-items: center;
    justify-content: center;
    border: 1px solid var(--color-border);
    border-radius: 50%;
    background: none;
    font-size: 12px;
    font-weight: 600;
    color: var(--color-text-muted);
    cursor: pointer;
    padding: 0;
    line-height: 1;
    flex-shrink: 0;
  }

  .about-btn:hover {
    border-color: var(--color-accent);
    color: var(--color-accent);
    background: none;
  }

  .toolbar-nav {
    display: flex;
    gap: 20px;
    align-items: center;
  }

  .btn-group {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .group-label {
    font-size: 11px;
    font-weight: 500;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--color-text-muted);
    margin-right: 2px;
  }

  .group-buttons {
    display: flex;
  }

  .group-buttons .toolbar-btn:first-child {
    border-top-right-radius: 0;
    border-bottom-right-radius: 0;
  }

  .group-buttons .toolbar-btn:last-child {
    border-top-left-radius: 0;
    border-bottom-left-radius: 0;
  }

  .group-buttons .toolbar-btn:not(:first-child):not(:last-child) {
    border-radius: 0;
  }

  .group-buttons .toolbar-btn:not(:first-child) {
    margin-left: -1px;
  }

  .group-buttons .toolbar-btn:only-child {
    border-radius: var(--radius);
  }

  .toolbar-btn {
    padding: 5px 12px;
    font-size: 13px;
    border: 1px solid var(--color-border);
    background: var(--color-surface);
    color: var(--color-text-muted);
    cursor: pointer;
    border-radius: var(--radius);
    transition: background 0.15s, color 0.15s, border-color 0.15s;
    position: relative;
  }

  .toolbar-btn:hover {
    background: var(--color-surface-raised);
    color: var(--color-text);
  }

  .toolbar-btn.active {
    background: var(--color-accent);
    color: white;
    border-color: var(--color-accent);
    z-index: 1;
  }

  .body-area {
    flex: 1;
    display: flex;
    overflow: hidden;
  }

  .content {
    flex: 1;
    overflow-y: auto;
    padding: 28px 32px;
  }

  .glossary-inline {
    flex-shrink: 0;
    overflow: hidden;
    height: 100%;
  }

  .glossary-overlay-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.15);
    z-index: 90;
  }

  .glossary-overlay {
    position: fixed;
    top: 0;
    right: 0;
    bottom: 0;
    z-index: 100;
    box-shadow: -4px 0 16px rgba(0, 0, 0, 0.1);
  }

  .settings-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.15);
    z-index: 90;
  }

  .settings-modal {
    position: fixed;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    z-index: 100;
    width: 560px;
    max-width: calc(100vw - 48px);
    max-height: 80vh;
    background: var(--color-surface);
    border-radius: 12px;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.12);
    overflow-y: auto;
  }
</style>
