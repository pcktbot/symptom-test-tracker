<script lang="ts">
  import '../app.css';
  import type { View } from '$lib/types';
  import Dashboard from '$lib/views/Dashboard.svelte';
  import LabResults from '$lib/views/LabResults.svelte';
  import LabEntry from '$lib/views/LabEntry.svelte';
  import Trends from '$lib/views/Trends.svelte';
  import SymptomEntry from '$lib/views/SymptomEntry.svelte';
  import Glossary from '$lib/views/Glossary.svelte';
  import Welcome from '$lib/views/Welcome.svelte';
  import Settings from '$lib/views/Settings.svelte';
  import Diagnoses from '$lib/views/Diagnoses.svelte';
  import Artifacts from '$lib/views/Artifacts.svelte';
  import Chat from '$lib/views/Chat.svelte';
  import LabManage from '$lib/views/LabManage.svelte';
  import SymptomEditor from '$lib/views/SymptomEditor.svelte';
  import Export from '$lib/views/Export.svelte';
  import { getSetting, setSetting } from '$lib/db';

  const welcomeSeen = typeof localStorage !== 'undefined' && localStorage.getItem('welcome_seen') === 'true';
  let currentView: View = $state(welcomeSeen ? 'dashboard' : 'welcome');
  let editSessionId: number | null = $state(null);
  let settingsOpen = $state(false);
  let glossaryOpen = $state(false);
  let glossaryTest: string | null = $state(null);
  let chatOpen = $state(false);
  let labConfigOpen = $state(false);
  let symptomConfigOpen = $state(false);
  let exportOpen = $state(false);
  let chatEnabled = $state(false);

  type FontSize = 'sm' | 'md' | 'lg';
  let fontSize: FontSize = $state('md');

  const CHAT_MIN_WIDTH = 240;
  const CHAT_MAX_WIDTH = 600;
  let chatWidth = $state(320);

  // Track body area width to decide inline vs overlay
  // We measure body-area (not content) to avoid resize loops when glossary toggles
  let bodyArea: HTMLElement | undefined = $state();
  let bodyWidth = $state(0);

  const GLOSSARY_WIDTH = 340;
  const CONTENT_MAX = 960; // largest max-width among views
  const CONFIG_PANEL_WIDTH = 400;
  const EXPORT_PANEL_WIDTH = 360;

  // Glossary fits inline when the body area is wide enough for both
  let glossaryInline = $derived(bodyWidth >= CONTENT_MAX + GLOSSARY_WIDTH + 48);
  let labConfigInline = $derived(bodyWidth >= CONTENT_MAX + CONFIG_PANEL_WIDTH + 48);
  let symptomConfigInline = $derived(bodyWidth >= CONTENT_MAX + CONFIG_PANEL_WIDTH + 48);
  let exportInline = $derived(bodyWidth >= CONTENT_MAX + EXPORT_PANEL_WIDTH + 48);

  function navigate(view: View, sessionId?: number | null) {
    currentView = view;
    editSessionId = sessionId ?? null;
  }

  function dismissWelcome() {
    localStorage.setItem('welcome_seen', 'true');
    currentView = 'dashboard';
  }

  function startChatResize(e: MouseEvent) {
    e.preventDefault();
    const startX = e.clientX;
    const startWidth = chatWidth;

    function onMove(e: MouseEvent) {
      const delta = startX - e.clientX;
      chatWidth = Math.max(CHAT_MIN_WIDTH, Math.min(CHAT_MAX_WIDTH, startWidth + delta));
    }

    function onUp() {
      window.removeEventListener('mousemove', onMove);
      window.removeEventListener('mouseup', onUp);
      setSetting('chat_width', String(chatWidth));
    }

    window.addEventListener('mousemove', onMove);
    window.addEventListener('mouseup', onUp);
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

  $effect(() => {
    Promise.all([
      getSetting('chat_enabled'),
      getSetting('chat_width'),
      getSetting('font_size'),
    ]).then(([chatEnabledVal, chatWidthVal, fontSizeVal]) => {
      chatEnabled = chatEnabledVal === 'true';
      const parsed = parseInt(chatWidthVal);
      if (!isNaN(parsed)) chatWidth = Math.max(CHAT_MIN_WIDTH, Math.min(CHAT_MAX_WIDTH, parsed));
      if (fontSizeVal === 'sm' || fontSizeVal === 'md' || fontSizeVal === 'lg') fontSize = fontSizeVal;
      document.body.dataset.fontSize = fontSize;
    });
  });

  $effect(() => {
    document.body.dataset.fontSize = fontSize;
  });

  $effect(() => {
    function handleKeydown(e: KeyboardEvent) {
      if (!e.metaKey) return;
      if (e.key === '=' || e.key === '+') {
        e.preventDefault();
        const next: FontSize = fontSize === 'sm' ? 'md' : fontSize === 'md' ? 'lg' : 'lg';
        fontSize = next;
        setSetting('font_size', fontSize);
      } else if (e.key === '-') {
        e.preventDefault();
        const next: FontSize = fontSize === 'lg' ? 'md' : fontSize === 'md' ? 'sm' : 'sm';
        fontSize = next;
        setSetting('font_size', fontSize);
      } else if (e.key === '0') {
        e.preventDefault();
        fontSize = 'md';
        setSetting('font_size', 'md');
      }
    }
    window.addEventListener('keydown', handleKeydown);
    return () => window.removeEventListener('keydown', handleKeydown);
  });

  type NavGroup = { label: string; items: { view: View; label: string }[] };

  const navGroups: NavGroup[] = [
    { label: 'Overview', items: [{ view: 'dashboard', label: 'Dashboard' }] },
    {
      label: 'Labs',
      items: [
        { view: 'lab-results', label: 'Results' },
        { view: 'trends', label: 'Trends' },
      ],
    },
    {
      label: 'Symptoms',
      items: [
        { view: 'symptoms', label: 'Log' },
      ],
    },
    {
      label: 'Data',
      items: [
        { view: 'artifacts', label: 'Artifacts' },
      ],
    },
    { label: 'Profile', items: [{ view: 'diagnoses', label: 'Diagnoses' }] },
  ];
</script>

<div class="app-layout">
  <header class="toolbar">
    <div class="toolbar-left">
      <div class="toolbar-title">Symptom Tracker</div>
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

      <div class="toolbar-nav-end">
        <button class="about-btn" onclick={() => navigate('welcome')} title="About this app">?</button>
        <button class="about-btn" onclick={() => settingsOpen = true} title="Settings">
          <svg width="12" height="12" viewBox="0 0 16 16" fill="currentColor">
            <path d="M8 4.754a3.246 3.246 0 1 0 0 6.492 3.246 3.246 0 0 0 0-6.492zM5.754 8a2.246 2.246 0 1 1 4.492 0 2.246 2.246 0 0 1-4.492 0z"/>
            <path d="M9.796 1.343c-.527-1.79-3.065-1.79-3.592 0l-.094.319a.873.873 0 0 1-1.255.52l-.292-.16c-1.64-.892-3.433.902-2.54 2.541l.159.292a.873.873 0 0 1-.52 1.255l-.319.094c-1.79.527-1.79 3.065 0 3.592l.319.094a.873.873 0 0 1 .52 1.255l-.16.292c-.892 1.64.902 3.434 2.541 2.54l.292-.159a.873.873 0 0 1 1.255.52l.094.319c.527 1.79 3.065 1.79 3.592 0l.094-.319a.873.873 0 0 1 1.255-.52l.292.16c1.64.893 3.434-.902 2.54-2.541l-.159-.292a.873.873 0 0 1 .52-1.255l.319-.094c1.79-.527 1.79-3.065 0-3.592l-.319-.094a.873.873 0 0 1-.52-1.255l.16-.292c.893-1.64-.902-3.433-2.541-2.54l-.292.159a.873.873 0 0 1-1.255-.52l-.094-.319zm-2.633.283c.246-.835 1.428-.835 1.674 0l.094.319a1.873 1.873 0 0 0 2.693 1.115l.291-.16c.764-.415 1.6.42 1.184 1.185l-.159.292a1.873 1.873 0 0 0 1.116 2.692l.318.094c.835.246.835 1.428 0 1.674l-.319.094a1.873 1.873 0 0 0-1.115 2.693l.16.291c.415.764-.421 1.6-1.185 1.184l-.291-.159a1.873 1.873 0 0 0-2.693 1.116l-.094.318c-.246.835-1.428.835-1.674 0l-.094-.319a1.873 1.873 0 0 0-2.692-1.115l-.292.16c-.764.415-1.6-.421-1.184-1.185l.159-.291A1.873 1.873 0 0 0 1.945 8.93l-.319-.094c-.835-.246-.835-1.428 0-1.674l.319-.094A1.873 1.873 0 0 0 3.06 4.377l-.16-.292c-.415-.764.421-1.6 1.185-1.184l.292.159a1.873 1.873 0 0 0 2.692-1.116l.094-.318z"/>
          </svg>
        </button>
        {#if chatEnabled}
          <button
            class="chat-icon-btn"
            class:active={chatOpen}
            onclick={() => chatOpen = !chatOpen}
            title="AI Assistant"
          >
            <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
              <path d="M2.678 11.894a1 1 0 0 1 .287.801 10.97 10.97 0 0 1-.398 2c1.395-.323 2.247-.697 2.634-.893a1 1 0 0 1 .71-.074A8.06 8.06 0 0 0 8 14c3.996 0 7-2.807 7-6 0-3.192-3.004-6-7-6S1 4.808 1 8c0 1.468.617 2.83 1.678 3.894zm-.493 3.905a21.682 21.682 0 0 1-.713.129c-.2.032-.352-.176-.273-.362a9.68 9.68 0 0 0 .244-.637l.003-.01c.248-.72.45-1.548.524-2.319C.743 11.37 0 9.76 0 8c0-3.866 3.582-7 8-7s8 3.134 8 7-3.582 7-8 7a9.06 9.06 0 0 1-2.347-.306c-.52.263-1.639.742-3.468 1.105z"/>
            </svg>
          </button>
        {/if}
      </div>
    </nav>
  </header>

  <div class="body-area" bind:this={bodyArea}>
    <main class="content">
      {#if currentView === 'welcome'}
        <Welcome onDismiss={dismissWelcome} />
      {:else if currentView === 'dashboard'}
        <Dashboard onNavigate={navigate} {openGlossary} />
      {:else if currentView === 'lab-results'}
        <LabResults onNavigate={navigate} {openGlossary} openLabConfig={() => labConfigOpen = true} />
      {:else if currentView === 'lab-entry'}
        <LabEntry sessionId={editSessionId} onNavigate={navigate} {openGlossary} />
      {:else if currentView === 'trends'}
        <Trends {openGlossary} onNavigate={navigate} openLabConfig={() => labConfigOpen = true} />
      {:else if currentView === 'symptoms'}
        <SymptomEntry onNavigate={navigate} openSymptomConfig={() => symptomConfigOpen = true} />
      {:else if currentView === 'artifacts'}
        <Artifacts onNavigate={navigate} openExport={() => exportOpen = true} />
      {:else if currentView === 'diagnoses'}
        <Diagnoses />
      {/if}
    </main>

    {#if glossaryOpen && glossaryInline}
      <div class="glossary-inline" style="width: {GLOSSARY_WIDTH}px">
        <Glossary activeTest={glossaryTest} onClose={closeGlossary} />
      </div>
    {/if}

    {#if chatOpen}
      <div class="chat-inline" style="width: {chatWidth}px">
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <div class="chat-resize-handle" onmousedown={startChatResize}></div>
        <Chat onClose={() => chatOpen = false} />
      </div>
    {/if}

    {#if labConfigOpen && labConfigInline}
      <div class="config-panel-inline" style="width: {CONFIG_PANEL_WIDTH}px">
        <LabManage onClose={() => labConfigOpen = false} />
      </div>
    {/if}

    {#if symptomConfigOpen && symptomConfigInline}
      <div class="config-panel-inline" style="width: {CONFIG_PANEL_WIDTH}px">
        <SymptomEditor onClose={() => symptomConfigOpen = false} />
      </div>
    {/if}

    {#if exportOpen && exportInline}
      <div class="config-panel-inline" style="width: {EXPORT_PANEL_WIDTH}px">
        <Export onClose={() => exportOpen = false} />
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

  {#if labConfigOpen && !labConfigInline}
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="panel-overlay-backdrop" onclick={() => labConfigOpen = false} onkeydown={() => {}}></div>
    <div class="panel-overlay" style="width: {CONFIG_PANEL_WIDTH}px">
      <LabManage onClose={() => labConfigOpen = false} />
    </div>
  {/if}

  {#if symptomConfigOpen && !symptomConfigInline}
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="panel-overlay-backdrop" onclick={() => symptomConfigOpen = false} onkeydown={() => {}}></div>
    <div class="panel-overlay" style="width: {CONFIG_PANEL_WIDTH}px">
      <SymptomEditor onClose={() => symptomConfigOpen = false} />
    </div>
  {/if}

  {#if exportOpen && !exportInline}
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="panel-overlay-backdrop" onclick={() => exportOpen = false} onkeydown={() => {}}></div>
    <div class="panel-overlay" style="width: {EXPORT_PANEL_WIDTH}px">
      <Export onClose={() => exportOpen = false} />
    </div>
  {/if}

  {#if settingsOpen}
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="settings-backdrop" onclick={() => settingsOpen = false} onkeydown={() => {}}></div>
    <div class="settings-modal">
      <Settings
        onClose={() => settingsOpen = false}
        {fontSize}
        onFontSizeChange={(size) => { fontSize = size; setSetting('font_size', size); }}
      />
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
    padding: 12px 24px;
    border-bottom: 1px solid var(--color-border);
    flex-shrink: 0;
  }

  .toolbar-left {
    display: flex;
    align-items: center;
    gap: 8px;
    padding-right: 16px;
    border-right: 1px solid var(--color-border);
    flex-shrink: 0;
  }

  .toolbar-title {
    font-size: 15px;
    font-weight: 600;
    white-space: nowrap;
  }

  .toolbar-nav {
    display: flex;
    gap: 20px;
    align-items: flex-end;
    flex: 1;
  }

  .toolbar-nav-end {
    margin-left: auto;
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .btn-group {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 4px;
  }

  .group-label {
    font-size: 10px;
    font-weight: 500;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--color-text-muted);
    padding-left: 2px;
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

  .about-btn {
    width: 28px;
    height: 28px;
    display: flex;
    align-items: center;
    justify-content: center;
    border: 1px solid var(--color-border);
    border-radius: var(--radius);
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

  .chat-inline {
    flex-shrink: 0;
    overflow: hidden;
    height: 100%;
    position: relative;
    border-left: 1px solid var(--color-border);
  }

  .chat-resize-handle {
    position: absolute;
    left: 0;
    top: 0;
    bottom: 0;
    width: 4px;
    cursor: col-resize;
    z-index: 10;
  }

  .chat-resize-handle:hover,
  .chat-resize-handle:active {
    background: var(--color-accent);
    opacity: 0.4;
  }

  .config-panel-inline {
    flex-shrink: 0;
    overflow-y: auto;
    height: 100%;
    border-left: 1px solid var(--color-border);
    background: var(--color-surface);
  }

  .panel-overlay-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.15);
    z-index: 90;
  }

  .panel-overlay {
    position: fixed;
    top: 0;
    right: 0;
    bottom: 0;
    z-index: 100;
    background: var(--color-surface);
    box-shadow: -4px 0 16px rgba(0, 0, 0, 0.1);
    overflow-y: auto;
  }

  .chat-icon-btn {
    width: 28px;
    height: 28px;
    display: flex;
    align-items: center;
    justify-content: center;
    border: 1px solid var(--color-border);
    border-radius: var(--radius);
    background: var(--color-surface);
    color: var(--color-text-muted);
    cursor: pointer;
    padding: 0;
    transition: background 0.15s, color 0.15s, border-color 0.15s;
  }

  .chat-icon-btn:hover {
    background: var(--color-surface-raised);
    color: var(--color-text);
  }

  .chat-icon-btn.active {
    background: var(--color-accent);
    color: white;
    border-color: var(--color-accent);
  }
</style>
