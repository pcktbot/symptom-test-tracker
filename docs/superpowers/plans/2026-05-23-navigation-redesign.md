# Navigation Redesign Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Demote Manage/Export to slide-in panels, restructure the toolbar, add resizable chat, persist font size and window dimensions.

**Architecture:** All panel state lives in `+page.svelte` alongside the existing Glossary pattern. Configure/Export views gain `onClose` props and are rendered as inline-or-overlay panels using the existing `bodyWidth` measurement. Toolbar is restructured in-place. Font size uses CSS `zoom` on `body`. Window dimensions use Tauri's `@tauri-apps/api/window` API from the frontend — no new Rust commands needed.

**Tech Stack:** Svelte 5 (runes), TypeScript, Tauri v2 (`@tauri-apps/api/window`), CSS custom properties.

**Note on tests:** This codebase's vitest suite covers `src/lib/utils.ts` utilities only. All tasks here are pure UI changes — verify each task manually by running `bun run dev` and testing in the app.

---

### Task 1: Clean up View type and navGroups

Remove `lab-manage`, `symptom-editor`, and `export` as navigable views. Remove them from the toolbar nav and view-switcher. The view files themselves stay on disk — they become panel components.

**Files:**
- Modify: `src/lib/types.ts`
- Modify: `src/routes/+page.svelte`

- [ ] **Step 1: Remove the three views from the View union type**

In `src/lib/types.ts`, find the line:
```ts
export type View = 'dashboard' | 'lab-results' | 'lab-entry' | 'trends' | 'symptoms' | 'symptom-editor' | 'export' | 'welcome' | 'lab-manage' | 'artifacts' | 'diagnoses';
```
Replace with:
```ts
export type View = 'dashboard' | 'lab-results' | 'lab-entry' | 'trends' | 'symptoms' | 'welcome' | 'artifacts' | 'diagnoses';
```

- [ ] **Step 2: Update navGroups in +page.svelte**

Replace the entire `navGroups` constant:
```ts
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
```

- [ ] **Step 3: Remove the three view-switcher branches from +page.svelte**

Remove these three `{:else if}` blocks:
```svelte
{:else if currentView === 'symptom-editor'}
  <SymptomEditor />
{:else if currentView === 'lab-manage'}
  <LabManage />
{:else if currentView === 'export'}
  <Export />
```

- [ ] **Step 4: Remove unused imports from +page.svelte**

Remove these import lines (they'll be re-imported as panel components in Task 3):
```ts
import SymptomEditor from '$lib/views/SymptomEditor.svelte';
import Export from '$lib/views/Export.svelte';
import LabManage from '$lib/views/LabManage.svelte';
```

- [ ] **Step 5: Run type check**

```bash
bun run check
```
Expected: no errors. If you see errors about `View` members, you removed something still referenced elsewhere — track it down with `grep -rn 'lab-manage\|symptom-editor\|export' src/`.

- [ ] **Step 6: Commit**

```bash
git add src/lib/types.ts src/routes/+page.svelte
git commit -m "refactor: remove manage/export from top-level nav views"
```

---

### Task 2: Add onClose prop to LabManage, SymptomEditor, and Export

Each of these views becomes a panel component. They need an `onClose` callback so they can render a close button, and to mirror the Glossary interface.

**Files:**
- Modify: `src/lib/views/LabManage.svelte`
- Modify: `src/lib/views/SymptomEditor.svelte`
- Modify: `src/lib/views/Export.svelte`

- [ ] **Step 1: Add onClose to LabManage**

In `src/lib/views/LabManage.svelte`, find the `<script>` block opening. Add a props declaration immediately after the existing imports (it currently has none):
```ts
let { onClose }: { onClose: () => void } = $props();
```

Then find the top-level container element in the template (look for `<div class="lab-manage">` or similar) and add a close button in the header. Find the `<h2>` or `<h1>` heading at the top of the template and wrap it in a header row:
```svelte
<div class="panel-header">
  <h2>Configure Labs</h2>
  <button class="close-btn" onclick={onClose} aria-label="Close">&times;</button>
</div>
```

Add these styles inside the `<style>` block:
```css
.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 20px 24px 16px;
  border-bottom: 1px solid var(--color-border);
  flex-shrink: 0;
}

.panel-header h2 {
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
```

- [ ] **Step 2: Add onClose to SymptomEditor**

In `src/lib/views/SymptomEditor.svelte`, add props declaration after imports:
```ts
let { onClose }: { onClose: () => void } = $props();
```

Add the same panel header markup and styles as in Step 1, with title "Configure Symptoms":
```svelte
<div class="panel-header">
  <h2>Configure Symptoms</h2>
  <button class="close-btn" onclick={onClose} aria-label="Close">&times;</button>
</div>
```

Add the same `.panel-header` and `.close-btn` styles to its `<style>` block.

- [ ] **Step 3: Add onClose to Export**

In `src/lib/views/Export.svelte`, add props declaration after imports:
```ts
let { onClose }: { onClose: () => void } = $props();
```

Add panel header with "Export Data" title and the same close button markup and styles.

- [ ] **Step 4: Run type check**

```bash
bun run check
```
Expected: no errors.

- [ ] **Step 5: Commit**

```bash
git add src/lib/views/LabManage.svelte src/lib/views/SymptomEditor.svelte src/lib/views/Export.svelte
git commit -m "refactor: add onClose prop to configure and export panel views"
```

---

### Task 3: Wire configure and export panels in +page.svelte

Add state and panel markup for LabManage, SymptomEditor, and Export — mirroring how the Glossary panel is handled.

**Files:**
- Modify: `src/routes/+page.svelte`

- [ ] **Step 1: Re-add imports for the three panel components**

In `src/routes/+page.svelte`, add back the imports (they were removed in Task 1):
```ts
import LabManage from '$lib/views/LabManage.svelte';
import SymptomEditor from '$lib/views/SymptomEditor.svelte';
import Export from '$lib/views/Export.svelte';
```

- [ ] **Step 2: Add panel state and constants**

After the existing `let glossaryOpen = $state(false);` line, add:
```ts
let labConfigOpen = $state(false);
let symptomConfigOpen = $state(false);
let exportOpen = $state(false);
```

After the existing `const CONTENT_MAX = 960;` and `const GLOSSARY_WIDTH = 340;` constants, add:
```ts
const CONFIG_PANEL_WIDTH = 400;
const EXPORT_PANEL_WIDTH = 360;
```

- [ ] **Step 3: Add derived inline-or-not booleans**

After the existing `let glossaryInline = $derived(...)` line, add:
```ts
let labConfigInline = $derived(bodyWidth >= CONTENT_MAX + CONFIG_PANEL_WIDTH + 48);
let symptomConfigInline = $derived(bodyWidth >= CONTENT_MAX + CONFIG_PANEL_WIDTH + 48);
let exportInline = $derived(bodyWidth >= CONTENT_MAX + EXPORT_PANEL_WIDTH + 48);
```

- [ ] **Step 4: Add inline panel markup inside .body-area**

In the template, inside the `<div class="body-area">` block, after the existing `glossary-inline` block, add:
```svelte
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
```

- [ ] **Step 5: Add overlay panel markup (below the body-area div, alongside the existing glossary overlay)**

After the existing glossary overlay block, add:
```svelte
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
```

- [ ] **Step 6: Add CSS for the new panel classes**

In `<style>`, add:
```css
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
```

- [ ] **Step 7: Run type check and verify in dev**

```bash
bun run check
```

Then start the app and confirm no JS errors. The panels aren't triggerable from the UI yet (that's Tasks 4–6) but the code should compile cleanly.

- [ ] **Step 8: Commit**

```bash
git add src/routes/+page.svelte
git commit -m "feat: add configure and export slide panels to app layout"
```

---

### Task 4: Add Configure button to LabResults

`LabResults` needs an `openLabConfig` callback prop. Add a gear+text "Configure" button to its header.

**Files:**
- Modify: `src/routes/+page.svelte`
- Modify: `src/lib/views/LabResults.svelte`

- [ ] **Step 1: Thread openLabConfig prop through page.svelte**

In the view-switcher block, find:
```svelte
{:else if currentView === 'lab-results'}
  <LabResults onNavigate={navigate} {openGlossary} />
```
Change to:
```svelte
{:else if currentView === 'lab-results'}
  <LabResults onNavigate={navigate} {openGlossary} openLabConfig={() => labConfigOpen = true} />
```

- [ ] **Step 2: Add openLabConfig to LabResults props**

In `src/lib/views/LabResults.svelte`, find the existing props declaration (line 8):
```ts
let { onNavigate, openGlossary }: { onNavigate: (view: View, sessionId?: number | null) => void; openGlossary: (testName?: string) => void } = $props();
```
Replace with:
```ts
let { onNavigate, openGlossary, openLabConfig }: {
  onNavigate: (view: View, sessionId?: number | null) => void;
  openGlossary: (testName?: string) => void;
  openLabConfig: () => void;
} = $props();
```

- [ ] **Step 3: Find the view's heading/header area and add the Configure button**

Locate the heading element near the top of the LabResults template (search for `<h1>` or `<h2>` or a `.view-header` class). Add a Configure button alongside it. If the heading is bare (e.g. `<h2>Lab Results</h2>`), wrap in a header row:
```svelte
<div class="view-header">
  <h2>Lab Results</h2>
  <button class="configure-btn" onclick={openLabConfig}>
    <svg width="14" height="14" viewBox="0 0 16 16" fill="currentColor">
      <path d="M8 4.754a3.246 3.246 0 1 0 0 6.492 3.246 3.246 0 0 0 0-6.492zM5.754 8a2.246 2.246 0 1 1 4.492 0 2.246 2.246 0 0 1-4.492 0z"/>
      <path d="M9.796 1.343c-.527-1.79-3.065-1.79-3.592 0l-.094.319a.873.873 0 0 1-1.255.52l-.292-.16c-1.64-.892-3.433.902-2.54 2.541l.159.292a.873.873 0 0 1-.52 1.255l-.319.094c-1.79.527-1.79 3.065 0 3.592l.319.094a.873.873 0 0 1 .52 1.255l-.16.292c-.892 1.64.902 3.434 2.541 2.54l.292-.159a.873.873 0 0 1 1.255.52l.094.319c.527 1.79 3.065 1.79 3.592 0l.094-.319a.873.873 0 0 1 1.255-.52l.292.16c1.64.893 3.434-.902 2.54-2.541l-.159-.292a.873.873 0 0 1 .52-1.255l.319-.094c1.79-.527 1.79-3.065 0-3.592l-.319-.094a.873.873 0 0 1-.52-1.255l.16-.292c.893-1.64-.902-3.433-2.541-2.54l-.292.159a.873.873 0 0 1-1.255-.52l-.094-.319zm-2.633.283c.246-.835 1.428-.835 1.674 0l.094.319a1.873 1.873 0 0 0 2.693 1.115l.291-.16c.764-.415 1.6.42 1.184 1.185l-.159.292a1.873 1.873 0 0 0 1.116 2.692l.318.094c.835.246.835 1.428 0 1.674l-.319.094a1.873 1.873 0 0 0-1.115 2.693l.16.291c.415.764-.421 1.6-1.185 1.184l-.291-.159a1.873 1.873 0 0 0-2.693 1.116l-.094.318c-.246.835-1.428.835-1.674 0l-.094-.319a1.873 1.873 0 0 0-2.692-1.115l-.292.16c-.764.415-1.6-.421-1.184-1.185l.159-.291A1.873 1.873 0 0 0 1.945 8.93l-.319-.094c-.835-.246-.835-1.428 0-1.674l.319-.094A1.873 1.873 0 0 0 3.06 4.377l-.16-.292c-.415-.764.421-1.6 1.185-1.184l.292.159a1.873 1.873 0 0 0 2.692-1.116l.094-.318z"/>
    </svg>
    Configure
  </button>
</div>
```

Add to the `<style>` block:
```css
.view-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 20px;
}

.view-header h2 {
  margin: 0;
}

.configure-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 12px;
  font-size: 13px;
  border: 1px solid var(--color-border);
  border-radius: var(--radius);
  background: var(--color-surface);
  color: var(--color-text-muted);
  cursor: pointer;
  transition: background 0.15s, color 0.15s, border-color 0.15s;
}

.configure-btn:hover {
  background: var(--color-surface-raised);
  color: var(--color-text);
  border-color: var(--color-border-strong);
}
```

- [ ] **Step 4: Run type check**

```bash
bun run check
```

- [ ] **Step 5: Verify in dev**

Run `bun run dev`, open Lab Results, click Configure — the LabManage panel should slide in from the right.

- [ ] **Step 6: Commit**

```bash
git add src/routes/+page.svelte src/lib/views/LabResults.svelte
git commit -m "feat: add Configure button to LabResults view"
```

---

### Task 5: Add Configure button to Trends

Same pattern as Task 4.

**Files:**
- Modify: `src/routes/+page.svelte`
- Modify: `src/lib/views/Trends.svelte`

- [ ] **Step 1: Thread openLabConfig through page.svelte**

Find:
```svelte
{:else if currentView === 'trends'}
  <Trends {openGlossary} onNavigate={navigate} />
```
Change to:
```svelte
{:else if currentView === 'trends'}
  <Trends {openGlossary} onNavigate={navigate} openLabConfig={() => labConfigOpen = true} />
```

- [ ] **Step 2: Add openLabConfig to Trends props**

In `src/lib/views/Trends.svelte`, find the existing props declaration (line 10):
```ts
let { openGlossary, onNavigate }: { openGlossary: (testName?: string) => void; onNavigate: (view: View) => void } = $props();
```
Replace with:
```ts
let { openGlossary, onNavigate, openLabConfig }: {
  openGlossary: (testName?: string) => void;
  onNavigate: (view: View) => void;
  openLabConfig: () => void;
} = $props();
```

- [ ] **Step 3: Add Configure button to Trends template**

Find the heading at the top of the Trends template and wrap in a `.view-header` row with the same Configure button markup from Task 4. Add the same `.view-header` and `.configure-btn` styles to the `<style>` block.

- [ ] **Step 4: Run type check and verify in dev**

```bash
bun run check
```

Open Trends, click Configure — LabManage panel opens.

- [ ] **Step 5: Commit**

```bash
git add src/routes/+page.svelte src/lib/views/Trends.svelte
git commit -m "feat: add Configure button to Trends view"
```

---

### Task 6: Add Configure button to SymptomEntry

**Files:**
- Modify: `src/routes/+page.svelte`
- Modify: `src/lib/views/SymptomEntry.svelte`

- [ ] **Step 1: Thread openSymptomConfig through page.svelte**

Find:
```svelte
{:else if currentView === 'symptoms'}
  <SymptomEntry onNavigate={navigate} />
```
Change to:
```svelte
{:else if currentView === 'symptoms'}
  <SymptomEntry onNavigate={navigate} openSymptomConfig={() => symptomConfigOpen = true} />
```

- [ ] **Step 2: Add openSymptomConfig to SymptomEntry props**

In `src/lib/views/SymptomEntry.svelte`, find:
```ts
let { onNavigate }: { onNavigate: (view: View) => void } = $props();
```
Replace with:
```ts
let { onNavigate, openSymptomConfig }: {
  onNavigate: (view: View) => void;
  openSymptomConfig: () => void;
} = $props();
```

- [ ] **Step 3: Add Configure button to SymptomEntry template**

Find the heading at the top of the template. Wrap in `.view-header` with the same Configure button. Add `.view-header` and `.configure-btn` styles.

- [ ] **Step 4: Run type check and verify in dev**

```bash
bun run check
```

Open Symptom Log, click Configure — SymptomEditor panel opens.

- [ ] **Step 5: Commit**

```bash
git add src/routes/+page.svelte src/lib/views/SymptomEntry.svelte
git commit -m "feat: add Configure button to SymptomEntry view"
```

---

### Task 7: Add Export button to Artifacts

**Files:**
- Modify: `src/routes/+page.svelte`
- Modify: `src/lib/views/Artifacts.svelte`

- [ ] **Step 1: Thread openExport through page.svelte**

Find:
```svelte
{:else if currentView === 'artifacts'}
  <Artifacts onNavigate={navigate} />
```
Change to:
```svelte
{:else if currentView === 'artifacts'}
  <Artifacts onNavigate={navigate} openExport={() => exportOpen = true} />
```

- [ ] **Step 2: Add openExport to Artifacts props**

In `src/lib/views/Artifacts.svelte`, find:
```ts
let { onNavigate }: { onNavigate: (view: View) => void } = $props();
```
Replace with:
```ts
let { onNavigate, openExport }: {
  onNavigate: (view: View) => void;
  openExport: () => void;
} = $props();
```

- [ ] **Step 3: Add Export button to Artifacts header**

Locate the top of the Artifacts template (near the heading/controls area). Add an Export button alongside existing controls:
```svelte
<button class="configure-btn" onclick={openExport}>
  <svg width="14" height="14" viewBox="0 0 16 16" fill="currentColor">
    <path d="M.5 9.9a.5.5 0 0 1 .5.5v2.5a1 1 0 0 0 1 1h12a1 1 0 0 0 1-1v-2.5a.5.5 0 0 1 1 0v2.5a2 2 0 0 1-2 2H2a2 2 0 0 1-2-2v-2.5a.5.5 0 0 1 .5-.5z"/>
    <path d="M7.646 11.854a.5.5 0 0 0 .708 0l3-3a.5.5 0 0 0-.708-.708L8.5 10.293V1.5a.5.5 0 0 0-1 0v8.793L5.354 8.146a.5.5 0 1 0-.708.708l3 3z"/>
  </svg>
  Export
</button>
```

Add the `.configure-btn` styles to the Artifacts `<style>` block (same rules as in Task 4).

- [ ] **Step 4: Run type check and verify in dev**

```bash
bun run check
```

Open Artifacts, click Export — Export panel opens.

- [ ] **Step 5: Commit**

```bash
git add src/routes/+page.svelte src/lib/views/Artifacts.svelte
git commit -m "feat: add Export button to Artifacts view"
```

---

### Task 8: Toolbar layout restructure

Move section labels above buttons (left-aligned), relocate Settings and About buttons to the right end of the nav (before chat), make chat icon-only (last item), keep only the app title in `toolbar-left`.

**Files:**
- Modify: `src/routes/+page.svelte`

- [ ] **Step 1: Update the toolbar HTML structure**

Replace the entire `<header class="toolbar">` block with:

```svelte
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
```

- [ ] **Step 2: Update toolbar styles**

Replace the existing toolbar-related CSS in `<style>` with:

```css
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
```

Remove the old `.chat-toggle-btn` style block entirely.

- [ ] **Step 3: Run type check and verify in dev**

```bash
bun run check
```

Run `bun run dev`. Confirm: section labels appear above buttons; About+Settings+Chat are at the far right; chat is icon-only.

- [ ] **Step 4: Commit**

```bash
git add src/routes/+page.svelte
git commit -m "feat: restructure toolbar layout with labels above buttons and end-aligned controls"
```

---

### Task 9: Chat drag-to-resize

Replace the hardcoded `320px` chat width with a draggable, persisted `chatWidth` state. A drag handle on the left edge of the panel lets the user resize it.

**Files:**
- Modify: `src/routes/+page.svelte`

- [ ] **Step 1: Replace chatWidth state and add load-from-settings**

In the `<script>` block, replace any hardcoded chat width reference. Find where `chatOpen` is declared and add after it:
```ts
const CHAT_MIN_WIDTH = 240;
const CHAT_MAX_WIDTH = 600;
let chatWidth = $state(320);
```

In the `$effect(() => { getSetting('chat_enabled')... })` block, extend it to also load chat width:
```ts
$effect(() => {
  Promise.all([
    getSetting('chat_enabled'),
    getSetting('chat_width'),
  ]).then(([chatEnabledVal, chatWidthVal]) => {
    chatEnabled = chatEnabledVal === 'true';
    const parsed = parseInt(chatWidthVal);
    if (!isNaN(parsed)) chatWidth = Math.max(CHAT_MIN_WIDTH, Math.min(CHAT_MAX_WIDTH, parsed));
  });
});
```

- [ ] **Step 2: Add drag handler functions**

In the `<script>` block, add:
```ts
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
```

Make sure `setSetting` is imported from `$lib/db`. Add it to the existing import:
```ts
import { getSetting, setSetting } from '$lib/db';
```

- [ ] **Step 3: Update chat panel markup**

Find the existing chat inline panel:
```svelte
{#if chatOpen}
  <div class="chat-inline" style="width: 320px">
    <Chat onClose={() => chatOpen = false} />
  </div>
{/if}
```

Replace with:
```svelte
{#if chatOpen}
  <div class="chat-inline" style="width: {chatWidth}px">
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="chat-resize-handle" onmousedown={startChatResize}></div>
    <Chat onClose={() => chatOpen = false} />
  </div>
{/if}
```

- [ ] **Step 4: Add drag handle styles**

In `<style>`, update `.chat-inline` and add `.chat-resize-handle`:
```css
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
```

- [ ] **Step 5: Run type check and verify in dev**

```bash
bun run check
```

Open the chat panel, drag the left edge — confirm it resizes. Close and reopen the app — confirm width is restored.

- [ ] **Step 6: Commit**

```bash
git add src/routes/+page.svelte
git commit -m "feat: add drag-to-resize for chat sidebar with persisted width"
```

---

### Task 10: Font size setting

Add `sm`/`md`/`lg` font scaling via CSS `zoom`, persisted to the `settings` table, with a control in Settings modal and `Cmd+`/`Cmd-`/`Cmd+0` keybindings.

**Files:**
- Modify: `src/app.css`
- Modify: `src/routes/+page.svelte`
- Modify: `src/lib/views/Settings.svelte`

- [ ] **Step 1: Add zoom rules to app.css**

In `src/app.css`, after the `:root` block, add:
```css
body[data-font-size="sm"] { zoom: 0.9; }
body[data-font-size="md"] { zoom: 1; }
body[data-font-size="lg"] { zoom: 1.15; }
```

- [ ] **Step 2: Add fontSize state and load from settings**

In `src/routes/+page.svelte` `<script>`, add:
```ts
type FontSize = 'sm' | 'md' | 'lg';
let fontSize: FontSize = $state('md');
```

Extend the existing settings load effect to also load `font_size`. If you consolidated the settings load in Task 9, add `font_size` there:
```ts
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
```

- [ ] **Step 3: Apply fontSize to body on change**

Add a reactive effect that keeps `body.dataset.fontSize` in sync:
```ts
$effect(() => {
  document.body.dataset.fontSize = fontSize;
});
```

- [ ] **Step 4: Add font size keyboard handler**

In `+page.svelte`, add a `keydown` listener. Find where the existing keyboard handling lives (or add a new `$effect`):
```ts
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
```

- [ ] **Step 5: Thread fontSize into Settings modal**

In `+page.svelte`, pass `fontSize` and an `onFontSizeChange` handler into the Settings component. Find the Settings modal usage:
```svelte
<Settings onClose={() => settingsOpen = false} />
```
Change to:
```svelte
<Settings
  onClose={() => settingsOpen = false}
  {fontSize}
  onFontSizeChange={(size) => { fontSize = size; setSetting('font_size', size); }}
/>
```

- [ ] **Step 6: Add font size control to Settings modal**

In `src/lib/views/Settings.svelte`, update the props declaration:
```ts
type FontSize = 'sm' | 'md' | 'lg';
let { onClose, fontSize = 'md', onFontSizeChange }: {
  onClose: () => void;
  fontSize?: FontSize;
  onFontSizeChange?: (size: FontSize) => void;
} = $props();
```

Add a new section to the Settings template, before the MCP section:
```svelte
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
```

Add to `<style>`:
```css
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
```

- [ ] **Step 7: Run type check and verify in dev**

```bash
bun run check
```

Open Settings, confirm Small/Medium/Large buttons appear and changing them rescales the UI. Press `Cmd+=` and `Cmd+-` in the main window — confirm scaling changes. Reopen app — confirm setting is restored.

- [ ] **Step 8: Commit**

```bash
git add src/app.css src/routes/+page.svelte src/lib/views/Settings.svelte
git commit -m "feat: add persisted font size setting with keyboard shortcuts"
```

---

### Task 11: Persist window dimensions

Save window size and position to the `settings` table on resize/move, and restore them on startup.

**Files:**
- Modify: `src/routes/+page.svelte`

- [ ] **Step 1: Add Tauri window API import**

In `+page.svelte` `<script>`, add:
```ts
import { getCurrentWindow, LogicalSize, LogicalPosition } from '@tauri-apps/api/window';
```

- [ ] **Step 2: Add window restore on mount**

Add a new `$effect` that runs once to restore saved window dimensions:
```ts
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
```

- [ ] **Step 3: Add debounced dimension save**

Add a debounce helper and a resize listener:
```ts
let saveDimensionsTimer: ReturnType<typeof setTimeout> | null = null;

function scheduleSaveDimensions() {
  if (saveDimensionsTimer) clearTimeout(saveDimensionsTimer);
  saveDimensionsTimer = setTimeout(async () => {
    const win = getCurrentWindow();
    const size = await win.innerSize();
    const pos = await win.innerPosition();
    setSetting('window_width', String(size.width));
    setSetting('window_height', String(size.height));
    setSetting('window_x', String(pos.x));
    setSetting('window_y', String(pos.y));
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
```

- [ ] **Step 4: Run type check**

```bash
bun run check
```

If `@tauri-apps/api/window` is not found, verify the package is available:
```bash
grep "@tauri-apps/api" package.json
```
It should be listed. If `LogicalSize` or `LogicalPosition` aren't exported from that path, try the import:
```ts
import { Window as TauriWindow } from '@tauri-apps/api/window';
const win = new TauriWindow('main');
```
and use `win.setSize(...)` / `win.innerSize()` etc. Check the actual exports with `bun run check` error messages.

- [ ] **Step 5: Verify in dev**

Run the full app (`bun run tauri dev`). Resize the window and wait half a second. Close and reopen — confirm it opens at the same size and position.

- [ ] **Step 6: Commit**

```bash
git add src/routes/+page.svelte
git commit -m "feat: persist and restore window dimensions across sessions"
```

---

## Self-Review

**Spec coverage check:**
- ✅ Toolbar: labels above buttons, settings/about at end, chat icon-only last → Task 8
- ✅ Lab Configure panel (LabManage) → Tasks 2, 3, 4, 5
- ✅ Symptom Configure panel (SymptomEditor) → Tasks 2, 3, 6
- ✅ Export panel from Artifacts → Tasks 2, 3, 7
- ✅ View type cleanup → Task 1
- ✅ Chat drag-to-resize → Task 9
- ✅ Font size (persist, CSS, keybindings, settings UI) → Task 10
- ✅ Window dimension persistence → Task 11

**Placeholder scan:** No TBD/TODO items. All code blocks are complete.

**Type consistency:**
- `FontSize` type defined in Task 10 Step 2, used consistently in Steps 4–6.
- `openLabConfig`, `openSymptomConfig`, `openExport` prop names consistent across page.svelte threading and view prop declarations.
- `labConfigOpen`, `symptomConfigOpen`, `exportOpen` state names consistent in panel markup.
- `CHAT_MIN_WIDTH`, `CHAT_MAX_WIDTH`, `chatWidth` consistent across Task 9.
- `CONFIG_PANEL_WIDTH`, `EXPORT_PANEL_WIDTH` defined in Task 3, used in Tasks 3 inline panels and overlay panels.
