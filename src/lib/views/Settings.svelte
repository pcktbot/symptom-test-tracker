<script lang="ts">
  import { onMount } from 'svelte';
  import { getSetting, setSetting } from '$lib/db';

  let { onClose }: { onClose: () => void } = $props();

  let mcpEnabled = $state(true);
  let loading = $state(true);
  let setupExpanded = $state(true);

  onMount(async () => {
    const val = await getSetting('mcp_enabled');
    mcpEnabled = val === 'true';
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
  }
</script>

<div class="settings">
  <div class="settings-header">
    <h2>Settings</h2>
    <button class="close-btn" onclick={onClose} aria-label="Close settings">&times;</button>
  </div>

  <div class="settings-body">
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
          <span class="toggle-subtitle">When enabled, AI assistants with the MCP server configured can query your lab results and symptom logs (read-only).</span>
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
          <p class="setup-desc">The MCP server gives AI assistants read-only access to your tracking data via the Model Context Protocol.</p>

          <h4>Available tools</h4>
          <ul class="tools-list">
            <li>Recent lab results (by date range)</li>
            <li>Abnormal lab values (latest flagged)</li>
            <li>Symptom history with severity</li>
            <li>Test trends over time</li>
            <li>Daily wellness summaries</li>
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
</style>
