<script lang="ts">
  import { onMount } from 'svelte';
  import { getSetting, setSetting, exportData } from '$lib/db';
  import { todayString } from '$lib/utils';
  import { save } from '@tauri-apps/plugin-dialog';
  import { writeTextFile } from '@tauri-apps/plugin-fs';
  import {
    PRESETS,
    DEFAULT_PRESET_ID,
    parseThemeSetting,
    serializeThemeSetting,
    applyTheme,
    type TokenKey,
    type ThemeSetting,
  } from '$lib/theme';

  type FontSize = 'sm' | 'md' | 'lg';
  let { onClose, fontSize = 'md', onFontSizeChange }: {
    onClose: () => void;
    fontSize?: FontSize;
    onFontSizeChange?: (size: FontSize) => void;
  } = $props();

  let theme: ThemeSetting = $state({ preset: DEFAULT_PRESET_ID, overrides: {} });
  let saveTimer: ReturnType<typeof setTimeout> | null = null;

  const TOKEN_GROUPS: { label: string; keys: TokenKey[] }[] = [
    { label: 'Surfaces', keys: ['--bg', '--surface', '--border'] },
    { label: 'Text', keys: ['--text', '--text-muted'] },
    { label: 'Brand', keys: ['--primary', '--primary-contrast'] },
    { label: 'Accents', keys: ['--accent-high', '--accent-low', '--accent-good', '--accent-bad'] },
  ];

  const TOKEN_LABELS: Record<TokenKey, string> = {
    '--bg': 'Background',
    '--surface': 'Card surface',
    '--border': 'Border',
    '--text': 'Text',
    '--text-muted': 'Muted text',
    '--primary': 'Primary',
    '--primary-contrast': 'Primary contrast',
    '--accent-high': 'High flag',
    '--accent-low': 'Low flag',
    '--accent-good': 'Good',
    '--accent-bad': 'Bad',
  };

  $effect(() => {
    getSetting('theme').then((raw) => {
      theme = parseThemeSetting(raw);
    });
  });

  function currentValue(key: TokenKey): string {
    if (theme.overrides[key]) return theme.overrides[key]!;
    const preset = PRESETS.find((p) => p.id === theme.preset) ?? PRESETS[0];
    return preset.tokens[key];
  }

  function scheduleSave() {
    applyTheme(document.documentElement, theme);
    if (saveTimer) clearTimeout(saveTimer);
    saveTimer = setTimeout(() => {
      setSetting('theme', serializeThemeSetting(theme));
    }, 200);
  }

  function selectPreset(id: string) {
    theme = { preset: id, overrides: {} };
    scheduleSave();
  }

  function setOverride(key: TokenKey, value: string) {
    theme = { ...theme, overrides: { ...theme.overrides, [key]: value } };
    scheduleSave();
  }

  function resetOverrides() {
    theme = { ...theme, overrides: {} };
    scheduleSave();
  }

  let mcpEnabled = $state(true);
  let mcpWriteEnabled = $state(false);
  let loading = $state(true);
  let setupExpanded = $state(true);

  onMount(async () => {
    const [readVal, writeVal] = await Promise.all([
      getSetting('mcp_enabled'),
      getSetting('mcp_write_enabled'),
    ]);
    mcpEnabled = readVal === 'true';
    mcpWriteEnabled = writeVal === 'true';
    loading = false;

    function handleKeydown(e: KeyboardEvent) {
      if (e.key === 'Escape') onClose();
    }
    window.addEventListener('keydown', handleKeydown);
    return () => window.removeEventListener('keydown', handleKeydown);
  });

  async function toggleMcp() {
    mcpEnabled = !mcpEnabled;
    await setSetting('mcp_enabled', mcpEnabled ? 'true' : 'false');
    if (!mcpEnabled && mcpWriteEnabled) {
      mcpWriteEnabled = false;
      await setSetting('mcp_write_enabled', 'false');
    }
  }

  async function toggleMcpWrite() {
    mcpWriteEnabled = !mcpWriteEnabled;
    await setSetting('mcp_write_enabled', mcpWriteEnabled ? 'true' : 'false');
  }

  let exporting = $state(false);
  let exportSuccess = $state(false);

  async function handleExport(format: 'json' | 'csv') {
    exporting = true;
    try {
      const oneYearAgo = (() => {
        const d = new Date();
        d.setFullYear(d.getFullYear() - 1);
        return d.toISOString().slice(0, 10);
      })();
      const data = await exportData(oneYearAgo, todayString(), true, true, format);
      const ext = format === 'json' ? 'json' : 'csv';
      const filePath = await save({
        defaultPath: `symptom-tracker-export.${ext}`,
        filters: [{ name: format === 'json' ? 'JSON' : 'CSV', extensions: [ext] }],
      });
      if (filePath) {
        await writeTextFile(filePath, data);
        exportSuccess = true;
        setTimeout(() => exportSuccess = false, 3000);
      }
    } catch (e) {
      console.error('Export failed:', e);
      alert('Export failed: ' + e);
    }
    exporting = false;
  }
</script>

<div class="settings">
  <div class="settings-header">
    <h2>Settings</h2>
    <button class="close-btn" onclick={onClose} aria-label="Close settings">&times;</button>
  </div>

  <div class="settings-body">
    <section class="settings-section">
      <h2>Theme</h2>

      <label class="row">
        <span>Preset</span>
        <select value={theme.preset} onchange={(e) => selectPreset((e.currentTarget as HTMLSelectElement).value)}>
          {#each PRESETS as preset}
            <option value={preset.id}>{preset.label}</option>
          {/each}
        </select>
      </label>

      {#each TOKEN_GROUPS as group}
        <div class="token-group">
          <div class="token-group-label">{group.label}</div>
          {#each group.keys as key}
            <label class="row">
              <span>{TOKEN_LABELS[key]}</span>
              <input
                type="color"
                value={currentValue(key)}
                oninput={(e) => setOverride(key, (e.currentTarget as HTMLInputElement).value)}
              />
            </label>
          {/each}
        </div>
      {/each}

      <button onclick={resetOverrides}>Reset to preset defaults</button>
    </section>

    <section class="settings-section">
      <h2>Export</h2>
      <p class="row-note">Download all your data.</p>
      <div class="button-row">
        <button class="btn-primary btn-sm" onclick={() => handleExport('json')} disabled={exporting}>
          {exporting ? 'Exporting…' : 'Export JSON'}
        </button>
        <button class="btn-primary btn-sm" onclick={() => handleExport('csv')} disabled={exporting}>
          {exporting ? 'Exporting…' : 'Export CSV'}
        </button>
        {#if exportSuccess}
          <span class="export-success">Saved successfully</span>
        {/if}
      </div>
    </section>

    <section class="section">
      <h3>Appearance</h3>
      <div class="field-label" style="margin-bottom: 8px;">Text size</div>
      <div class="font-size-options">
        {#each (['sm', 'md', 'lg'] as const) as size}
          <button
            class="font-size-btn"
            class:active={fontSize === size}
            onclick={() => onFontSizeChange?.(size)}
          >
            {size === 'sm' ? 'Small' : size === 'md' ? 'Medium' : 'Large'}
          </button>
        {/each}
      </div>
      <span class="toggle-subtitle" style="margin-top: 6px; display: block;">
        Also: ⌘= / ⌘- to resize, ⌘0 to reset
      </span>
    </section>


    <section class="section">
      <h3>MCP Access</h3>
      <div class="toggle-row">
        <button
          class="toggle"
          class:on={mcpEnabled}
          onclick={toggleMcp}
          disabled={loading}
          role="switch"
          aria-checked={mcpEnabled}
          aria-label="Toggle MCP access"
        >
          <span class="toggle-knob"></span>
        </button>
        <div class="toggle-label">
          <span class="toggle-title">Allow MCP clients to read your data</span>
          <span class="toggle-subtitle">When enabled, AI assistants with the MCP server configured can query your lab results and symptom logs.</span>
        </div>
      </div>

      <div class="toggle-row" style="margin-top: 12px;">
        <button
          class="toggle"
          class:on={mcpWriteEnabled}
          onclick={toggleMcpWrite}
          disabled={loading || !mcpEnabled}
          role="switch"
          aria-checked={mcpWriteEnabled}
          aria-label="Toggle MCP write access"
        >
          <span class="toggle-knob"></span>
        </button>
        <div class="toggle-label">
          <span class="toggle-title">Allow MCP clients to write data</span>
          <span class="toggle-subtitle">When enabled, AI assistants can insert new lab sessions and results (e.g. from a pasted lab report).</span>
        </div>
      </div>
    </section>

    <section class="section">
      <button class="section-toggle" onclick={() => setupExpanded = !setupExpanded}>
        <h3>MCP Setup Instructions</h3>
        <span class="chevron" class:expanded={setupExpanded}></span>
      </button>

      {#if setupExpanded}
        <div class="setup-content">
          <p class="setup-desc">The MCP server gives AI assistants access to your tracking data via the Model Context Protocol. Read access lets them query data; write access lets them insert new lab results.</p>

          <h4>Available tools</h4>
          <ul class="tools-list">
            <li>Recent lab results (by date range)</li>
            <li>Abnormal lab values (latest flagged)</li>
            <li>Symptom history with severity</li>
            <li>Test trends over time</li>
            <li>Daily wellness summaries</li>
            <li>Insert lab session with results (requires write access)</li>
          </ul>

          <h4>Binary path</h4>
          <p class="setup-note">After building, the server binary is at:</p>
          <code class="code-block">target/release/mcp-server</code>

          <h4>Claude Desktop</h4>
          <p class="setup-note">Add to <code>~/Library/Application Support/Claude/claude_desktop_config.json</code>:</p>
          <pre class="code-block">{`{
  "mcpServers": {
    "symptom-tracker": {
      "command": "/absolute/path/to/mcp-server"
    }
  }
}`}</pre>

          <h4>Claude Code</h4>
          <p class="setup-note">Add to <code>.mcp.json</code> in your project root:</p>
          <pre class="code-block">{`{
  "mcpServers": {
    "symptom-tracker": {
      "command": "/absolute/path/to/mcp-server"
    }
  }
}`}</pre>

          <h4>Disabling access</h4>
          <p class="setup-note">Toggle the switch above to block all MCP clients at once, or remove the config entry from individual clients.</p>
        </div>
      {/if}
    </section>
  </div>
</div>

<style>
  .settings {
    display: flex;
    flex-direction: column;
    height: 100%;
  }

  .settings-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 20px 24px 16px;
    border-bottom: 1px solid var(--color-border);
  }

  .settings-header h2 {
    margin: 0;
  }

  .close-btn {
    width: 28px;
    height: 28px;
    display: flex;
    align-items: center;
    justify-content: center;
    border: none;
    background: none;
    font-size: 20px;
    color: var(--color-text-muted);
    cursor: pointer;
    border-radius: 4px;
    padding: 0;
  }

  .close-btn:hover {
    background: var(--color-surface-raised);
    color: var(--color-text);
  }

  .settings-body {
    padding: 20px 24px;
    overflow-y: auto;
    flex: 1;
  }

  .section {
    margin-bottom: 24px;
  }

  .section h3 {
    margin: 0 0 12px;
  }

  .toggle-row {
    display: flex;
    align-items: flex-start;
    gap: 12px;
  }

  .toggle {
    position: relative;
    width: 44px;
    height: 24px;
    border-radius: 12px;
    border: 1px solid var(--color-border-strong);
    background: var(--color-surface-raised);
    cursor: pointer;
    padding: 0;
    flex-shrink: 0;
    transition: background 0.2s, border-color 0.2s;
  }

  .toggle.on {
    background: var(--color-accent);
    border-color: var(--color-accent);
  }

  .toggle-knob {
    position: absolute;
    top: 2px;
    left: 2px;
    width: 18px;
    height: 18px;
    border-radius: 50%;
    background: white;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.15);
    transition: transform 0.2s;
  }

  .toggle.on .toggle-knob {
    transform: translateX(20px);
  }

  .toggle-label {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .toggle-title {
    font-size: 14px;
    font-weight: 500;
  }

  .toggle-subtitle {
    font-size: 12px;
    color: var(--color-text-muted);
    line-height: 1.4;
  }

  .section-toggle {
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: 100%;
    background: none;
    border: none;
    padding: 0 0 12px;
    cursor: pointer;
  }

  .section-toggle:hover {
    background: none;
  }

  .section-toggle h3 {
    margin: 0;
  }

  .chevron {
    display: inline-block;
    width: 8px;
    height: 8px;
    border-right: 2px solid var(--color-text-muted);
    border-bottom: 2px solid var(--color-text-muted);
    transform: rotate(-45deg);
    transition: transform 0.2s;
  }

  .chevron.expanded {
    transform: rotate(45deg);
  }

  .setup-content {
    font-size: 13px;
    line-height: 1.5;
  }

  .setup-desc {
    margin-bottom: 16px;
    color: var(--color-text-muted);
  }

  .setup-content h4 {
    font-size: 13px;
    font-weight: 600;
    margin: 16px 0 4px;
  }

  .setup-content h4:first-child {
    margin-top: 0;
  }

  .tools-list {
    margin: 4px 0 0 16px;
    color: var(--color-text-muted);
    font-size: 12px;
  }

  .tools-list li {
    margin-bottom: 2px;
  }

  .setup-note {
    color: var(--color-text-muted);
    margin-bottom: 6px;
  }

  .setup-note code {
    font-family: var(--font-mono);
    font-size: 12px;
    background: var(--color-surface-raised);
    padding: 1px 4px;
    border-radius: 3px;
  }

  .code-block {
    display: block;
    font-family: var(--font-mono);
    font-size: 12px;
    background: var(--color-surface-raised);
    border: 1px solid var(--color-border);
    border-radius: var(--radius);
    padding: 10px 12px;
    overflow-x: auto;
    white-space: pre;
    margin: 4px 0 8px;
  }

  .field-label {
    font-size: 12px;
    font-weight: 500;
    color: var(--color-text-muted);
    display: block;
    margin-bottom: 4px;
  }

  .btn-primary {
    background: var(--color-accent);
    color: white;
    border: none;
    border-radius: var(--radius);
    padding: 6px 12px;
    font-size: 13px;
    font-weight: 500;
    cursor: pointer;
    transition: background 0.2s;
  }

  .btn-primary:hover {
    background: var(--color-accent-hover);
  }

  .btn-sm {
    padding: 6px 12px;
  }

  .font-size-options {
    display: flex;
    gap: 0;
  }

  .font-size-btn {
    flex: 1;
    padding: 6px 0;
    font-size: 13px;
    border: 1px solid var(--color-border);
    background: var(--color-surface);
    color: var(--color-text-muted);
    cursor: pointer;
    transition: background 0.15s, color 0.15s;
  }

  .font-size-btn:first-child {
    border-radius: var(--radius) 0 0 var(--radius);
  }

  .font-size-btn:last-child {
    border-radius: 0 var(--radius) var(--radius) 0;
    margin-left: -1px;
  }

  .font-size-btn:not(:first-child):not(:last-child) {
    border-radius: 0;
    margin-left: -1px;
  }

  .font-size-btn.active {
    background: var(--color-accent);
    color: white;
    border-color: var(--color-accent);
    z-index: 1;
    position: relative;
  }

  .settings-section {
    padding: 16px 0;
    border-bottom: 1px solid var(--border);
    margin-bottom: 24px;
  }

  .settings-section h2 {
    font-family: var(--font-heading);
    font-size: 16px;
    margin-bottom: 12px;
    margin-top: 0;
  }

  .row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 6px 0;
  }

  .row span {
    color: var(--text-muted, var(--color-text-muted));
    font-size: 13px;
  }

  .token-group {
    margin: 12px 0;
  }

  .token-group-label {
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--text-muted, var(--color-text-muted));
    margin: 6px 0;
  }

  .row-note { color: var(--text-muted); font-size: 13px; margin-bottom: 8px; }
  .button-row { display: flex; gap: 8px; align-items: center; }

  .export-success {
    color: var(--color-success);
    font-size: 13px;
    font-weight: 500;
  }
</style>
