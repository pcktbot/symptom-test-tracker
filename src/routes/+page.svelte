<script lang="ts">
  import '../app.css';
  import type { View } from '$lib/types';
  import Dashboard from '$lib/views/Dashboard.svelte';
  import LabResults from '$lib/views/LabResults.svelte';
  import LabEntry from '$lib/views/LabEntry.svelte';
  import Trends from '$lib/views/Trends.svelte';
  import DailyRating from '$lib/views/DailyRating.svelte';
  import Glossary from '$lib/views/Glossary.svelte';
  import Welcome from '$lib/views/Welcome.svelte';
  import Settings from '$lib/views/Settings.svelte';
  import CareTeam from '$lib/views/CareTeam.svelte';
  import Documents from '$lib/views/Documents.svelte';
  import LabManage from '$lib/views/LabManage.svelte';
  import { getSetting, setSetting } from '$lib/db';
  import { applyTheme, parseThemeSetting } from '$lib/theme';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { LogicalSize, LogicalPosition } from '@tauri-apps/api/dpi';

  const welcomeSeen = typeof localStorage !== 'undefined' && localStorage.getItem('welcome_seen') === 'true';
  let currentView: View = $state(welcomeSeen ? 'dashboard' : 'welcome');
  let editSessionId: number | null = $state(null);
  let settingsOpen = $state(false);
  let glossaryOpen = $state(false);
  let glossaryTest: string | null = $state(null);
  type ActivePanel = 'labConfig' | null;
  let activePanel: ActivePanel = $state(null);

  type FontSize = 'sm' | 'md' | 'lg';
  let fontSize: FontSize = $state('md');

  let saveDimensionsTimer: ReturnType<typeof setTimeout> | null = null;

  // Track body area width to decide inline vs overlay
  // We measure body-area (not content) to avoid resize loops when glossary toggles
  let bodyArea: HTMLElement | undefined = $state();
  let bodyWidth = $state(0);

  const GLOSSARY_WIDTH = 340;
  const CONTENT_MAX = 960; // largest max-width among views
  const CONFIG_PANEL_WIDTH = 400;

  // Glossary fits inline when the body area is wide enough for both
  let glossaryInline = $derived(bodyWidth >= CONTENT_MAX + GLOSSARY_WIDTH + 48);
  let configPanelInline = $derived(bodyWidth >= CONTENT_MAX + CONFIG_PANEL_WIDTH + 48);

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

  $effect(() => {
    getSetting('font_size').then((fontSizeVal) => {
      if (fontSizeVal === 'sm' || fontSizeVal === 'md' || fontSizeVal === 'lg') fontSize = fontSizeVal;
      document.body.dataset.fontSize = fontSize;
    });
  });

  $effect(() => {
    getSetting('theme').then((raw) => {
      const setting = parseThemeSetting(raw);
      applyTheme(document.documentElement, setting);
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

  $effect(() => {
    Promise.all([
      getSetting('window_width'),
      getSetting('window_height'),
      getSetting('window_x'),
      getSetting('window_y'),
    ]).then(([w, h, x, y]) => {
      const win = getCurrentWindow();
      const width = parseInt(w);
      const height = parseInt(h);
      const posX = parseInt(x);
      const posY = parseInt(y);
      if (!isNaN(width) && !isNaN(height) && width >= 900 && height >= 600) {
        win.setSize(new LogicalSize(width, height));
      }
      if (!isNaN(posX) && !isNaN(posY)) {
        win.setPosition(new LogicalPosition(posX, posY));
      }
    });
  });

  function scheduleSaveDimensions() {
    if (saveDimensionsTimer) clearTimeout(saveDimensionsTimer);
    saveDimensionsTimer = setTimeout(async () => {
      const win = getCurrentWindow();
      const factor = await win.scaleFactor();
      const size = (await win.innerSize()).toLogical(factor);
      const pos = (await win.innerPosition()).toLogical(factor);
      setSetting('window_width', String(Math.round(size.width)));
      setSetting('window_height', String(Math.round(size.height)));
      setSetting('window_x', String(Math.round(pos.x)));
      setSetting('window_y', String(Math.round(pos.y)));
    }, 500);
  }

  $effect(() => {
    const win = getCurrentWindow();
    const unlisten = win.listen('tauri://resize', scheduleSaveDimensions);
    const unlistenMove = win.listen('tauri://move', scheduleSaveDimensions);
    return () => {
      unlisten.then(f => f());
      unlistenMove.then(f => f());
    };
  });

  const NAV: { view: View; label: string }[] = [
    { view: 'dashboard', label: 'Dashboard' },
    { view: 'daily-rating', label: 'Daily Log' },
    { view: 'lab-results', label: 'Results' },
    { view: 'trends', label: 'Trends' },
    { view: 'documents', label: 'Documents' },
    { view: 'care-team', label: 'Care Team' },
  ];

  function logToday() {
    navigate('daily-rating');
  }

  function bumpFontSize(direction: 1 | -1) {
    const order: FontSize[] = ['sm', 'md', 'lg'];
    const idx = order.indexOf(fontSize);
    const next = order[Math.max(0, Math.min(order.length - 1, idx + direction))];
    if (next !== fontSize) {
      fontSize = next;
      setSetting('font_size', fontSize);
    }
  }
</script>

<div class="app-layout">
  <header class="topbar">
    <div class="brand">
      <div class="brand-tile"></div>
      <div class="brand-name">Baseline</div>
    </div>

    <nav class="pill-nav">
      {#each NAV as item}
        <button
          class="pill"
          class:active={currentView === item.view}
          onclick={() => navigate(item.view)}
        >
          {item.label}
        </button>
      {/each}
    </nav>

    <div class="topbar-actions">
      <button class="primary log-today" onclick={logToday}>+ Log today</button>
      <div class="font-size-group">
        <button class="icon-btn font-btn" onclick={() => bumpFontSize(-1)} disabled={fontSize === 'sm'} title="Smaller text (⌘−)">A−</button>
        <button class="icon-btn font-btn" onclick={() => bumpFontSize(1)} disabled={fontSize === 'lg'} title="Larger text (⌘+)">A+</button>
      </div>
      <button class="icon-btn" onclick={() => openGlossary()} title="Help">?</button>
      <button class="icon-btn" onclick={() => settingsOpen = true} title="Settings">
        <svg width="14" height="14" viewBox="0 0 16 16" fill="currentColor">
          <path d="M8 4.754a3.246 3.246 0 1 0 0 6.492 3.246 3.246 0 0 0 0-6.492zM5.754 8a2.246 2.246 0 1 1 4.492 0 2.246 2.246 0 0 1-4.492 0z"/>
          <path d="M9.796 1.343c-.527-1.79-3.065-1.79-3.592 0l-.094.319a.873.873 0 0 1-1.255.52l-.292-.16c-1.64-.892-3.433.902-2.54 2.541l.159.292a.873.873 0 0 1-.52 1.255l-.319.094c-1.79.527-1.79 3.065 0 3.592l.319.094a.873.873 0 0 1 .52 1.255l-.16.292c-.892 1.64.902 3.434 2.541 2.54l.292-.159a.873.873 0 0 1 1.255.52l.094.319c.527 1.79 3.065 1.79 3.592 0l.094-.319a.873.873 0 0 1 1.255-.52l.292.16c1.64.893 3.434-.902 2.54-2.541l-.159-.292a.873.873 0 0 1 .52-1.255l.319-.094c1.79-.527 1.79-3.065 0-3.592l-.319-.094a.873.873 0 0 1-.52-1.255l.16-.292c.893-1.64-.902-3.433-2.541-2.54l-.292.159a.873.873 0 0 1-1.255-.52l-.094-.319zm-2.633.283c.246-.835 1.428-.835 1.674 0l.094.319a1.873 1.873 0 0 0 2.693 1.115l.291-.16c.764-.415 1.6.42 1.184 1.185l-.159.292a1.873 1.873 0 0 0 1.116 2.692l.318.094c.835.246.835 1.428 0 1.674l-.319.094a1.873 1.873 0 0 0-1.115 2.693l.16.291c.415.764-.421 1.6-1.185 1.184l-.291-.159a1.873 1.873 0 0 0-2.693 1.116l-.094.318c-.246.835-1.428.835-1.674 0l-.094-.319a1.873 1.873 0 0 0-2.692-1.115l-.292.16c-.764.415-1.6-.421-1.184-1.185l.159-.291A1.873 1.873 0 0 0 1.945 8.93l-.319-.094c-.835-.246-.835-1.428 0-1.674l.319-.094A1.873 1.873 0 0 0 3.06 4.377l-.16-.292c-.415-.764.421-1.6 1.185-1.184l.292.159a1.873 1.873 0 0 0 2.692-1.116l.094-.318z"/>
        </svg>
      </button>
    </div>
  </header>

  <div class="body-area" bind:this={bodyArea}>
    <main class="content">
      {#if currentView === 'welcome'}
        <Welcome onDismiss={dismissWelcome} />
      {:else if currentView === 'dashboard'}
        <Dashboard onNavigate={navigate} {openGlossary} />
      {:else if currentView === 'lab-results'}
        <LabResults onNavigate={navigate} {openGlossary} openLabConfig={() => activePanel = 'labConfig'} />
      {:else if currentView === 'lab-entry'}
        <LabEntry sessionId={editSessionId} onNavigate={navigate} {openGlossary} />
      {:else if currentView === 'trends'}
        <Trends {openGlossary} onNavigate={navigate} openLabConfig={() => activePanel = 'labConfig'} />
      {:else if currentView === 'daily-rating'}
        <DailyRating />
      {:else if currentView === 'documents'}
        <Documents onNavigate={navigate} />
      {:else if currentView === 'care-team'}
        <CareTeam />
      {/if}
    </main>

    {#if glossaryOpen && glossaryInline}
      <div class="glossary-inline" style="width: {GLOSSARY_WIDTH}px">
        <Glossary activeTest={glossaryTest} onClose={closeGlossary} />
      </div>
    {/if}

    {#if activePanel === 'labConfig' && configPanelInline}
      <div class="config-panel-inline" style="width: {CONFIG_PANEL_WIDTH}px">
        <LabManage onClose={() => activePanel = null} />
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

  {#if activePanel === 'labConfig' && !configPanelInline}
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="panel-overlay-backdrop" onclick={() => activePanel = null} onkeydown={() => {}}></div>
    <div class="panel-overlay" style="width: {CONFIG_PANEL_WIDTH}px">
      <LabManage onClose={() => activePanel = null} />
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

  .topbar {
    display: grid;
    grid-template-columns: 1fr auto 1fr;
    align-items: center;
    gap: 16px;
    padding: 14px 24px;
    background: var(--bg);
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }

  .brand {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .brand-tile {
    width: 26px;
    height: 26px;
    background: var(--primary);
    border-radius: 6px;
  }

  .brand-name {
    font-family: var(--font-heading);
    font-weight: 700;
    font-size: 18px;
    color: var(--text);
  }

  .pill-nav {
    display: flex;
    gap: 6px;
    justify-content: center;
  }

  .pill {
    padding: 6px 14px;
    border-radius: 999px;
    border: 1px solid transparent;
    background: transparent;
    color: var(--text);
    font-size: 13px;
    font-weight: 500;
    cursor: pointer;
    transition: background 0.15s, color 0.15s;
  }

  .pill:hover {
    background: var(--surface);
  }

  .pill.active {
    background: var(--primary);
    color: var(--primary-contrast);
    border-color: var(--primary);
  }

  .topbar-actions {
    display: flex;
    align-items: center;
    gap: 8px;
    justify-content: flex-end;
  }

  .log-today {
    padding: 6px 14px;
    border-radius: 8px;
    font-size: 13px;
    font-weight: 600;
  }

  .icon-btn {
    width: 32px;
    height: 32px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    border-radius: 8px;
    border: 1px solid var(--border);
    background: var(--surface);
    color: var(--text-muted);
    padding: 0;
    cursor: pointer;
  }

  .icon-btn:hover {
    color: var(--text);
    border-color: var(--primary);
  }

  .icon-btn:disabled {
    opacity: 0.4;
    cursor: default;
  }

  .icon-btn:disabled:hover {
    color: var(--text-muted);
    border-color: var(--border);
  }

  .font-size-group { display: flex; gap: 4px; }

  .font-btn {
    width: auto;
    padding: 0 8px;
    font-size: 12px;
    font-weight: 700;
    font-family: var(--font-heading);
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


</style>
