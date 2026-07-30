# Baseline: Theming & View Reorganization Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Rebrand the app to "Baseline", replace the sidebar with a top pill-nav matching the mockup, redesign the Dashboard, restyle existing views, and introduce a user-editable theme (color tokens + presets, Lato heading font) that removes the need for a separate dark mode. Appointments, Providers, and Documents ship as visual placeholders with seeded demo data.

**Architecture:** All colors migrated to CSS custom properties on `:root`. A new `src/lib/theme.ts` module defines presets and applies token overrides read from the existing `settings` table under key `theme`. Fonts remain bundled locally (Lato added alongside existing IBM Plex). No Rust command changes.

**Tech Stack:** Svelte 5 (runes), Tauri v2, existing `settings` get/set commands, `@fontsource/lato` (already npm-installed if needed — otherwise self-hosted `.woff2` in `static/fonts/`), vitest for unit tests on `theme.ts`.

**Spec:** `docs/superpowers/specs/2026-07-30-baseline-theming-design.md`

---

## Phase 1 — Theming foundation

### Task 1: Define theme token defaults and presets module

**Files:**
- Create: `src/lib/theme.ts`
- Create: `src/lib/theme.test.ts`

- [ ] **Step 1: Write the failing tests**

```ts
// src/lib/theme.test.ts
import { describe, it, expect } from 'vitest';
import { PRESETS, DEFAULT_PRESET_ID, resolveTheme, TOKEN_KEYS } from './theme';

describe('theme presets', () => {
  it('exposes the expected preset ids', () => {
    expect(PRESETS.map((p) => p.id)).toEqual(['baseline-warm', 'cool-slate', 'high-contrast']);
  });

  it('every preset defines all token keys', () => {
    for (const preset of PRESETS) {
      for (const key of TOKEN_KEYS) {
        expect(preset.tokens[key], `${preset.id} missing ${key}`).toBeTruthy();
      }
    }
  });

  it('defaults to baseline-warm', () => {
    expect(DEFAULT_PRESET_ID).toBe('baseline-warm');
  });
});

describe('resolveTheme', () => {
  it('returns preset tokens when there are no overrides', () => {
    const resolved = resolveTheme({ preset: 'baseline-warm', overrides: {} });
    const preset = PRESETS.find((p) => p.id === 'baseline-warm')!;
    expect(resolved).toEqual(preset.tokens);
  });

  it('applies overrides on top of preset tokens', () => {
    const resolved = resolveTheme({
      preset: 'baseline-warm',
      overrides: { '--primary': '#ff0000' },
    });
    expect(resolved['--primary']).toBe('#ff0000');
    expect(resolved['--bg']).toBe(PRESETS[0].tokens['--bg']);
  });

  it('falls back to default preset when preset id is unknown', () => {
    const resolved = resolveTheme({ preset: 'nope', overrides: {} });
    expect(resolved['--bg']).toBe(PRESETS[0].tokens['--bg']);
  });
});
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `bun run test:run src/lib/theme.test.ts`
Expected: FAIL (module does not exist).

- [ ] **Step 3: Implement the module**

```ts
// src/lib/theme.ts
export const TOKEN_KEYS = [
  '--bg',
  '--surface',
  '--border',
  '--text',
  '--text-muted',
  '--primary',
  '--primary-contrast',
  '--accent-high',
  '--accent-low',
  '--accent-good',
  '--accent-bad',
] as const;

export type TokenKey = (typeof TOKEN_KEYS)[number];
export type TokenMap = Record<TokenKey, string>;

export interface Preset {
  id: string;
  label: string;
  tokens: TokenMap;
}

export interface ThemeSetting {
  preset: string;
  overrides: Partial<TokenMap>;
}

export const PRESETS: Preset[] = [
  {
    id: 'baseline-warm',
    label: 'Baseline Warm',
    tokens: {
      '--bg': '#f5f0e8',
      '--surface': '#fbfaf6',
      '--border': '#e6ded0',
      '--text': '#1a1a1a',
      '--text-muted': '#7a7368',
      '--primary': '#1f3a3a',
      '--primary-contrast': '#ffffff',
      '--accent-high': '#b8763a',
      '--accent-low': '#3a5878',
      '--accent-good': '#5a7a3a',
      '--accent-bad': '#8a2f2f',
    },
  },
  {
    id: 'cool-slate',
    label: 'Cool Slate',
    tokens: {
      '--bg': '#eef1f5',
      '--surface': '#ffffff',
      '--border': '#d9dee6',
      '--text': '#1c2530',
      '--text-muted': '#5f6b7a',
      '--primary': '#2b3a67',
      '--primary-contrast': '#ffffff',
      '--accent-high': '#c47b2a',
      '--accent-low': '#2f5da8',
      '--accent-good': '#3f8054',
      '--accent-bad': '#a53434',
    },
  },
  {
    id: 'high-contrast',
    label: 'High Contrast',
    tokens: {
      '--bg': '#ffffff',
      '--surface': '#ffffff',
      '--border': '#000000',
      '--text': '#000000',
      '--text-muted': '#333333',
      '--primary': '#000000',
      '--primary-contrast': '#ffffff',
      '--accent-high': '#a35a00',
      '--accent-low': '#003a99',
      '--accent-good': '#005a1a',
      '--accent-bad': '#a30000',
    },
  },
];

export const DEFAULT_PRESET_ID = 'baseline-warm';

export function resolveTheme(setting: ThemeSetting): TokenMap {
  const preset = PRESETS.find((p) => p.id === setting.preset) ?? PRESETS[0];
  return { ...preset.tokens, ...(setting.overrides as TokenMap) };
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `bun run test:run src/lib/theme.test.ts`
Expected: PASS (3 preset tests + 3 resolveTheme tests).

- [ ] **Step 5: Commit**

```bash
git add src/lib/theme.ts src/lib/theme.test.ts
git commit -m "feat(theme): add theme tokens, presets, resolveTheme"
```

---

### Task 2: Add applyTheme + load/save helpers

**Files:**
- Modify: `src/lib/theme.ts`
- Modify: `src/lib/theme.test.ts`

- [ ] **Step 1: Add failing tests**

Append to `src/lib/theme.test.ts`:

```ts
import { applyTheme, parseThemeSetting, serializeThemeSetting } from './theme';

describe('applyTheme', () => {
  it('sets every token as an inline style on the given element', () => {
    const el = document.createElement('div');
    applyTheme(el, { preset: 'baseline-warm', overrides: {} });
    expect(el.style.getPropertyValue('--primary')).toBe('#1f3a3a');
    expect(el.style.getPropertyValue('--bg')).toBe('#f5f0e8');
  });

  it('applies overrides', () => {
    const el = document.createElement('div');
    applyTheme(el, { preset: 'baseline-warm', overrides: { '--primary': '#ff0000' } });
    expect(el.style.getPropertyValue('--primary')).toBe('#ff0000');
  });
});

describe('parseThemeSetting', () => {
  it('returns default when input is empty', () => {
    expect(parseThemeSetting('')).toEqual({ preset: 'baseline-warm', overrides: {} });
  });

  it('returns default when input is invalid JSON', () => {
    expect(parseThemeSetting('not json')).toEqual({ preset: 'baseline-warm', overrides: {} });
  });

  it('returns parsed setting', () => {
    const s = JSON.stringify({ preset: 'cool-slate', overrides: { '--primary': '#111' } });
    expect(parseThemeSetting(s)).toEqual({ preset: 'cool-slate', overrides: { '--primary': '#111' } });
  });

  it('drops unknown override keys', () => {
    const s = JSON.stringify({ preset: 'baseline-warm', overrides: { '--primary': '#111', '--bogus': '#000' } });
    expect(parseThemeSetting(s).overrides).toEqual({ '--primary': '#111' });
  });
});

describe('serializeThemeSetting', () => {
  it('round-trips through parseThemeSetting', () => {
    const s: ThemeSetting = { preset: 'cool-slate', overrides: { '--bg': '#000' } };
    expect(parseThemeSetting(serializeThemeSetting(s))).toEqual(s);
  });
});
```

- [ ] **Step 2: Run tests to verify failure**

Run: `bun run test:run src/lib/theme.test.ts`
Expected: FAIL — imports not found.

- [ ] **Step 3: Implement**

Append to `src/lib/theme.ts`:

```ts
export function applyTheme(el: HTMLElement, setting: ThemeSetting): void {
  const tokens = resolveTheme(setting);
  for (const key of TOKEN_KEYS) {
    el.style.setProperty(key, tokens[key]);
  }
}

const DEFAULT_SETTING: ThemeSetting = { preset: DEFAULT_PRESET_ID, overrides: {} };

export function parseThemeSetting(raw: string): ThemeSetting {
  if (!raw) return { ...DEFAULT_SETTING };
  try {
    const parsed = JSON.parse(raw) as Partial<ThemeSetting>;
    const preset = typeof parsed.preset === 'string' ? parsed.preset : DEFAULT_PRESET_ID;
    const rawOverrides = (parsed.overrides ?? {}) as Record<string, string>;
    const overrides: Partial<TokenMap> = {};
    for (const key of TOKEN_KEYS) {
      if (typeof rawOverrides[key] === 'string') overrides[key] = rawOverrides[key];
    }
    return { preset, overrides };
  } catch {
    return { ...DEFAULT_SETTING };
  }
}

export function serializeThemeSetting(setting: ThemeSetting): string {
  return JSON.stringify(setting);
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `bun run test:run src/lib/theme.test.ts`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/lib/theme.ts src/lib/theme.test.ts
git commit -m "feat(theme): add applyTheme + settings serialization helpers"
```

---

### Task 3: Wire theme loader into app root

**Files:**
- Modify: `src/routes/+page.svelte:108-120` (existing settings-loader `$effect`) and top imports
- Modify: `src/app.css:57-78` (replace `:root` block)

- [ ] **Step 1: Update `:root` tokens in `src/app.css`**

Replace lines 57–78 with:

```css
:root {
  /* Baseline Warm preset — overridden at runtime by theme setting */
  --bg: #f5f0e8;
  --surface: #fbfaf6;
  --border: #e6ded0;
  --text: #1a1a1a;
  --text-muted: #7a7368;
  --primary: #1f3a3a;
  --primary-contrast: #ffffff;
  --accent-high: #b8763a;
  --accent-low: #3a5878;
  --accent-good: #5a7a3a;
  --accent-bad: #8a2f2f;

  /* Legacy aliases — retained until per-view restyles land */
  --color-text: var(--text);
  --color-text-muted: var(--text-muted);
  --color-border: var(--border);
  --color-border-strong: var(--border);
  --color-surface: var(--surface);
  --color-surface-raised: var(--bg);
  --color-accent: var(--primary);
  --color-accent-hover: var(--primary);
  --color-flag-high: var(--accent-high);
  --color-flag-low: var(--accent-low);
  --color-flag-critical: var(--accent-bad);
  --color-flag-normal: var(--accent-good);
  --color-success: var(--accent-good);
  --color-warning: var(--accent-high);
  --color-danger: var(--accent-bad);

  --font-sans: 'IBM Plex Sans', system-ui, sans-serif;
  --font-heading: 'Lato', 'IBM Plex Sans', system-ui, sans-serif;
  --font-mono: 'IBM Plex Mono', ui-monospace, monospace;
  --radius: 6px;
  --radius-lg: 12px;
  --shadow-sm: 0 1px 2px rgba(0,0,0,0.05);
  --shadow: 0 1px 4px rgba(0,0,0,0.08);
}

body { background: var(--bg); }
h1, h2, h3, h4, h5, h6 { font-family: var(--font-heading); }
```

- [ ] **Step 2: Add theme load to `+page.svelte`**

Add to the existing imports at the top of the script block:

```ts
import { applyTheme, parseThemeSetting, DEFAULT_PRESET_ID } from '$lib/theme';
```

Locate the effect at lines 108–120 (`Promise.all([ getSetting('chat_enabled'), ... ])`) and add a new sibling effect immediately after it:

```ts
$effect(() => {
  getSetting('theme').then((raw) => {
    const setting = parseThemeSetting(raw);
    applyTheme(document.documentElement, setting);
  });
});
```

- [ ] **Step 3: Manual verification**

Run: `bun run tauri dev`
Expected: App still boots. Background is warm cream (`#f5f0e8`). Existing controls still function (legacy aliases route to new tokens).

- [ ] **Step 4: Type check**

Run: `bun run check`
Expected: no errors.

- [ ] **Step 5: Commit**

```bash
git add src/app.css src/routes/+page.svelte
git commit -m "feat(theme): apply theme tokens at :root and load from settings"
```

---

### Task 4: Bundle Lato font

**Files:**
- Create: `static/fonts/Lato-Regular.woff2`
- Create: `static/fonts/Lato-Bold.woff2`
- Create: `static/fonts/Lato-Black.woff2`
- Modify: `src/app.css:1-56` (append `@font-face` blocks for Lato)

- [ ] **Step 1: Download Lato woff2 files**

Run:

```bash
mkdir -p static/fonts && curl -sSfL -o static/fonts/Lato-Regular.woff2 https://cdn.jsdelivr.net/fontsource/fonts/lato@latest/latin-400-normal.woff2 && curl -sSfL -o static/fonts/Lato-Bold.woff2 https://cdn.jsdelivr.net/fontsource/fonts/lato@latest/latin-700-normal.woff2 && curl -sSfL -o static/fonts/Lato-Black.woff2 https://cdn.jsdelivr.net/fontsource/fonts/lato@latest/latin-900-normal.woff2
```

Expected: three files present, each > 20KB.

- [ ] **Step 2: Add @font-face blocks**

Append to `src/app.css` after the last existing `@font-face` block (currently line 55):

```css
@font-face {
  font-family: 'Lato';
  src: url('/fonts/Lato-Regular.woff2') format('woff2');
  font-weight: 400;
  font-style: normal;
  font-display: swap;
}

@font-face {
  font-family: 'Lato';
  src: url('/fonts/Lato-Bold.woff2') format('woff2');
  font-weight: 700;
  font-style: normal;
  font-display: swap;
}

@font-face {
  font-family: 'Lato';
  src: url('/fonts/Lato-Black.woff2') format('woff2');
  font-weight: 900;
  font-style: normal;
  font-display: swap;
}
```

- [ ] **Step 3: Manual verification**

Run: `bun run tauri dev`
In DevTools → Network, verify `Lato-Regular.woff2` loads with status 200 and headings render in Lato.

- [ ] **Step 4: Commit**

```bash
git add static/fonts/Lato-Regular.woff2 static/fonts/Lato-Bold.woff2 static/fonts/Lato-Black.woff2 src/app.css
git commit -m "feat(theme): bundle Lato as heading font"
```

---

## Phase 2 — Nav shell

### Task 5: Add new nav item types & routing scaffolding

**Files:**
- Modify: `src/lib/types.ts:87` (extend `View` union)

- [ ] **Step 1: Extend View union**

Replace line 87 with:

```ts
export type View =
  | 'dashboard'
  | 'lab-results'
  | 'lab-entry'
  | 'trends'
  | 'daily-rating'
  | 'documents'
  | 'care-team'
  | 'welcome';
```

Remove `'artifacts'` and `'diagnoses'` from the union (they're consumed by `'documents'` and `'care-team'`).

- [ ] **Step 2: Fix downstream type errors**

Run: `bun run check`
Expected: errors listing each site that still passes `'artifacts'` or `'diagnoses'`.

For each reported location in `src/routes/+page.svelte`, update view branches:
- `{:else if currentView === 'artifacts'}` → `{:else if currentView === 'documents'}`
- `{:else if currentView === 'diagnoses'}` → `{:else if currentView === 'care-team'}`

Re-run `bun run check` — expect zero errors.

- [ ] **Step 3: Commit**

```bash
git add src/lib/types.ts src/routes/+page.svelte
git commit -m "refactor: rename view ids to documents/care-team"
```

---

### Task 6: Replace toolbar markup with Baseline top bar

**Files:**
- Modify: `src/routes/+page.svelte:194-266` (navGroups + toolbar markup)
- Modify: `src/routes/+page.svelte:354-500` (styles)

- [ ] **Step 1: Replace navGroups with flat nav array**

Replace lines 194–218 (the `navGroups` block) with:

```ts
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
```

- [ ] **Step 2: Replace `<header class="toolbar">` markup**

Replace lines 222–266 (entire `<header class="toolbar">…</header>`) with:

```svelte
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
    <button class="icon-btn" onclick={() => openGlossary()} title="Help">?</button>
    <button class="icon-btn" onclick={() => settingsOpen = true} title="Settings">
      <svg width="14" height="14" viewBox="0 0 16 16" fill="currentColor">
        <path d="M8 4.754a3.246 3.246 0 1 0 0 6.492 3.246 3.246 0 0 0 0-6.492zM5.754 8a2.246 2.246 0 1 1 4.492 0 2.246 2.246 0 0 1-4.492 0z"/>
        <path d="M9.796 1.343c-.527-1.79-3.065-1.79-3.592 0l-.094.319a.873.873 0 0 1-1.255.52l-.292-.16c-1.64-.892-3.433.902-2.54 2.541l.159.292a.873.873 0 0 1-.52 1.255l-.319.094c-1.79.527-1.79 3.065 0 3.592l.319.094a.873.873 0 0 1 .52 1.255l-.16.292c-.892 1.64.902 3.434 2.541 2.54l.292-.159a.873.873 0 0 1 1.255.52l.094.319c.527 1.79 3.065 1.79 3.592 0l.094-.319a.873.873 0 0 1 1.255-.52l.292.16c1.64.893 3.434-.902 2.54-2.541l-.159-.292a.873.873 0 0 1 .52-1.255l.319-.094c1.79-.527 1.79-3.065 0-3.592l-.319-.094a.873.873 0 0 1-.52-1.255l.16-.292c.893-1.64-.902-3.433-2.541-2.54l-.292.159a.873.873 0 0 1-1.255-.52l-.094-.319zm-2.633.283c.246-.835 1.428-.835 1.674 0l.094.319a1.873 1.873 0 0 0 2.693 1.115l.291-.16c.764-.415 1.6.42 1.184 1.185l-.159.292a1.873 1.873 0 0 0 1.116 2.692l.318.094c.835.246.835 1.428 0 1.674l-.319.094a1.873 1.873 0 0 0-1.115 2.693l.16.291c.415.764-.421 1.6-1.185 1.184l-.291-.159a1.873 1.873 0 0 0-2.693 1.116l-.094.318c-.246.835-1.428.835-1.674 0l-.094-.319a1.873 1.873 0 0 0-2.692-1.115l-.292.16c-.764.415-1.6-.421-1.184-1.185l.159-.291A1.873 1.873 0 0 0 1.945 8.93l-.319-.094c-.835-.246-.835-1.428 0-1.674l.319-.094A1.873 1.873 0 0 0 3.06 4.377l-.16-.292c-.415-.764.421-1.6 1.185-1.184l.292.159a1.873 1.873 0 0 0 2.692-1.116l.094-.318z"/>
      </svg>
    </button>
  </div>
</header>
```

- [ ] **Step 3: Replace `<style>` toolbar styles**

Inside the `<style>` block (starts at line 354), replace the toolbar-related rules (`.toolbar`, `.toolbar-left`, `.toolbar-title`, `.toolbar-nav`, `.toolbar-nav-end`, `.btn-group`, `.group-label`, `.group-buttons`, `.toolbar-btn`, `.about-btn`, `.chat-icon-btn` and their variants) with:

```css
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
  padding: 8px 18px;
  border-radius: 999px;
  border: 1px solid transparent;
  background: transparent;
  color: var(--text);
  font-size: 14px;
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
  padding: 8px 16px;
  border-radius: 8px;
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
```

Delete the `.chat-icon-btn` block entirely.

- [ ] **Step 4: Remove chat icon logic**

In the script block:
- Remove the `let chatEnabled = $state(false);` line and its assignment inside the `getSetting('chat_enabled')` effect.
- Remove all rendering of the Chat surface: delete the `{#if chatOpen}` inline block (lines ~295–301), delete the `chatOpen` state, `chatWidth` state, `startChatResize`, all `chat_*` settings reads/writes, and the `<Chat>` import.
- Delete the `.chat-inline`, `.chat-resize-handle` style blocks.
- Leave `Chat.svelte` and `chat.rs` on disk untouched.

- [ ] **Step 5: Type check + visual verification**

Run: `bun run check` — expect zero errors.
Run: `bun run tauri dev`
Expected: Top bar shows brand tile + "Baseline" + centered pills + `+ Log today` + `?` + gear. Sidebar gone. Clicking pills swaps views. `+ Log today` navigates to Daily Log. Chat icon gone.

- [ ] **Step 6: Commit**

```bash
git add src/routes/+page.svelte
git commit -m "feat(nav): replace toolbar with Baseline top bar and pill nav"
```

---

## Phase 3 — Settings reorganization + Theme panel

### Task 7: Add Theme section to Settings

**Files:**
- Modify: `src/lib/views/Settings.svelte`

- [ ] **Step 1: Read current Settings.svelte to locate insertion point**

Run: `wc -l src/lib/views/Settings.svelte`

Note the file length; identify a spot at the end of the settings sections (before the closing `</section>` or `</div>`) to insert the Theme block.

- [ ] **Step 2: Add Theme UI**

Add imports at the top of `<script>`:

```ts
import { PRESETS, TOKEN_KEYS, DEFAULT_PRESET_ID, parseThemeSetting, serializeThemeSetting, applyTheme, type TokenKey, type ThemeSetting } from '$lib/theme';
import { getSetting, setSetting } from '$lib/db';

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
```

Add markup inside the Settings body (near the top so it's discoverable):

```svelte
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
```

Add styles inside the `<style>` block:

```css
.settings-section { padding: 16px 0; border-bottom: 1px solid var(--border); }
.settings-section h2 { font-family: var(--font-heading); font-size: 16px; margin-bottom: 12px; }
.row { display: flex; align-items: center; justify-content: space-between; padding: 6px 0; }
.row span { color: var(--text-muted); font-size: 13px; }
.token-group { margin: 12px 0; }
.token-group-label {
  font-size: 11px; font-weight: 600; text-transform: uppercase;
  letter-spacing: 0.06em; color: var(--text-muted); margin: 6px 0;
}
```

- [ ] **Step 3: Manual verification**

Run: `bun run tauri dev`
Open Settings (gear icon). Confirm the Theme section appears at top with preset dropdown + grouped color pickers. Switch preset — background/surface colors change immediately across the app. Edit a color — change is live. Close and reopen the app — selection persists.

- [ ] **Step 4: Commit**

```bash
git add src/lib/views/Settings.svelte
git commit -m "feat(settings): add Theme panel with presets and per-token overrides"
```

---

### Task 8: Move Export into Settings

**Files:**
- Modify: `src/lib/views/Settings.svelte`
- Modify: `src/routes/+page.svelte` (remove Export panel wiring)
- Modify: `src/lib/views/Artifacts.svelte` (remove `openExport` prop calls)

- [ ] **Step 1: Inline Export UI into Settings**

Open `src/lib/views/Export.svelte`, note the export button(s) it renders (typically calls `invoke('export_json' | 'export_csv')` via `$lib/db`).

In `Settings.svelte`, add a new section after Theme:

```svelte
<section class="settings-section">
  <h2>Export</h2>
  <p class="row-note">Download all your data as JSON or CSV.</p>
  <div class="button-row">
    <button onclick={handleExportJson}>Export JSON</button>
    <button onclick={handleExportCsv}>Export CSV</button>
  </div>
</section>
```

In the script, add stubs that call the same commands `Export.svelte` currently calls. If `Export.svelte` uses `exportJson()` and `exportCsv()` from `$lib/db`, import and call those directly. If it uses a Tauri dialog to save, use `@tauri-apps/plugin-dialog` `save` + `@tauri-apps/plugin-fs` `writeTextFile` the same way `Export.svelte` does — copy the exact call sites verbatim (do not simplify).

```ts
async function handleExportJson() {
  // Copy the JSON export flow from Export.svelte verbatim.
}
async function handleExportCsv() {
  // Copy the CSV export flow from Export.svelte verbatim.
}
```

Add styles:

```css
.row-note { color: var(--text-muted); font-size: 13px; margin-bottom: 8px; }
.button-row { display: flex; gap: 8px; }
```

- [ ] **Step 2: Remove Export from Artifacts + toolbar wiring**

In `src/routes/+page.svelte`:
- Remove the `Export` import.
- Remove `activePanel === 'export'` branches (inline + overlay).
- Remove `openExport={() => activePanel = 'export'}` from the `<Artifacts>` render.
- Change `type ActivePanel = 'labConfig' | 'export' | null;` to `type ActivePanel = 'labConfig' | null;`.
- Remove `EXPORT_PANEL_WIDTH`, `exportPanelInline`.

In `src/lib/views/Artifacts.svelte`:
- Remove the `openExport` prop from the component's `$props()` and any button that called it.

- [ ] **Step 3: Type check + visual verification**

Run: `bun run check` — zero errors.
Run: `bun run tauri dev` — open Settings, verify Export buttons work end-to-end (a save dialog appears and produces a file).

- [ ] **Step 4: Commit**

```bash
git add src/lib/views/Settings.svelte src/lib/views/Artifacts.svelte src/routes/+page.svelte
git commit -m "feat(settings): move Export into Settings; drop side panel"
```

---

## Phase 4 — Dashboard rewrite

### Task 9: Seed demo data module for Upcoming Care, Providers, Appointments

**Files:**
- Create: `src/lib/demo.ts`
- Create: `src/lib/demo.test.ts`

- [ ] **Step 1: Write failing tests**

```ts
// src/lib/demo.test.ts
import { describe, it, expect } from 'vitest';
import { DEMO_APPOINTMENTS, DEMO_PROVIDERS, upcomingCare } from './demo';

describe('demo data', () => {
  it('exposes appointments with expected shape', () => {
    expect(DEMO_APPOINTMENTS.length).toBeGreaterThan(0);
    for (const a of DEMO_APPOINTMENTS) {
      expect(a.date).toMatch(/^\d{4}-\d{2}-\d{2}$/);
      expect(typeof a.title).toBe('string');
      expect(typeof a.location).toBe('string');
    }
  });

  it('exposes providers with initials derived from name', () => {
    expect(DEMO_PROVIDERS.length).toBeGreaterThan(0);
    for (const p of DEMO_PROVIDERS) {
      expect(p.initials).toHaveLength(2);
      expect(typeof p.specialty).toBe('string');
    }
  });

  it('upcomingCare returns the first N appointments sorted by date', () => {
    const items = upcomingCare(3);
    expect(items).toHaveLength(3);
    for (let i = 1; i < items.length; i++) {
      expect(items[i - 1].date <= items[i].date).toBe(true);
    }
  });
});
```

- [ ] **Step 2: Run to verify fail**

Run: `bun run test:run src/lib/demo.test.ts`
Expected: FAIL (module not found).

- [ ] **Step 3: Implement**

```ts
// src/lib/demo.ts
export interface DemoAppointment {
  date: string; // YYYY-MM-DD
  title: string;
  location: string;
}

export interface DemoProvider {
  name: string;
  specialty: string;
  initials: string;
}

export const DEMO_APPOINTMENTS: DemoAppointment[] = [
  { date: '2026-08-05', title: 'Saphnelo infusion', location: 'Oklahoma Arthritis Center' },
  { date: '2026-08-12', title: 'Rheumatology follow-up', location: 'Dr. Whitfield' },
  { date: '2026-09-02', title: 'CBC + metabolic panel', location: 'Diagnostic Lab of Oklahoma' },
];

export const DEMO_PROVIDERS: DemoProvider[] = [
  { name: 'Dr. Whitfield', specialty: 'Rheumatologist', initials: 'RW' },
  { name: 'Dr. Kim', specialty: 'Hematologist', initials: 'JK' },
];

export function upcomingCare(limit: number): DemoAppointment[] {
  return [...DEMO_APPOINTMENTS].sort((a, b) => a.date.localeCompare(b.date)).slice(0, limit);
}
```

- [ ] **Step 4: Run to verify pass**

Run: `bun run test:run src/lib/demo.test.ts`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/lib/demo.ts src/lib/demo.test.ts
git commit -m "feat(demo): seed appointments and providers for Care Team + Dashboard"
```

---

### Task 10: Rewrite Dashboard.svelte

**Files:**
- Modify: `src/lib/views/Dashboard.svelte` (full rewrite)

- [ ] **Step 1: Inspect existing Dashboard interface**

Run: `head -40 src/lib/views/Dashboard.svelte` — note the prop signature (uses `onNavigate: (view: View) => void` and `openGlossary`).

- [ ] **Step 2: Write new Dashboard**

Replace entire file contents:

```svelte
<script lang="ts">
  import type { View, Diagnosis, DailyRating, AbnormalResult } from '$lib/types';
  import { listDiagnoses, getRatingsRange, getAbnormalLabs } from '$lib/db';
  import { getHeatmapConfig } from '$lib/db';
  import { upcomingCare } from '$lib/demo';

  let { onNavigate }: { onNavigate: (view: View) => void; openGlossary?: (test?: string) => void } = $props();

  let diagnoses: Diagnosis[] = $state([]);
  let ratings: DailyRating[] = $state([]);
  let abnormals: AbnormalResult[] = $state([]);
  let heatmapColors: string[] = $state([]);
  let heatmapLabels: string[] = $state([]);

  const today = new Date();
  const dateStr = today.toISOString().slice(0, 10);

  function fmtLongDate(d: Date): string {
    return d.toLocaleDateString(undefined, { weekday: 'long', year: 'numeric', month: 'long', day: 'numeric' });
  }
  function greeting(d: Date): string {
    const h = d.getHours();
    if (h < 12) return 'Good morning';
    if (h < 18) return 'Good afternoon';
    return 'Good evening';
  }

  $effect(() => {
    listDiagnoses().then((rows) => { diagnoses = rows.filter((d) => !d.resolution_date); });
    getHeatmapConfig().then((c) => { heatmapColors = c.colors; heatmapLabels = c.labels; });
    getAbnormalLabs(5).then((rows) => { abnormals = rows; });
    const start = new Date(today);
    start.setDate(start.getDate() - 6);
    getRatingsRange(start.toISOString().slice(0, 10), dateStr).then((rows) => { ratings = rows; });
  });

  const last7 = $derived.by(() => {
    const map = new Map(ratings.map((r) => [r.log_date, r]));
    const out: { date: string; label: string; rating: DailyRating | null }[] = [];
    const dow = ['S', 'M', 'T', 'W', 'T', 'F', 'S'];
    for (let i = 6; i >= 0; i--) {
      const d = new Date(today);
      d.setDate(d.getDate() - i);
      const iso = d.toISOString().slice(0, 10);
      out.push({ date: iso, label: dow[d.getDay()], rating: map.get(iso) ?? null });
    }
    return out;
  });

  const todayRating = $derived(last7[last7.length - 1]?.rating ?? null);
  const fallbackRating = $derived.by(() => {
    if (todayRating) return todayRating;
    for (let i = last7.length - 2; i >= 0; i--) if (last7[i].rating) return last7[i].rating;
    return null;
  });
  const wellnessLabel = $derived(fallbackRating ? (heatmapLabels[fallbackRating.wellness_score - 1] ?? '') : 'Not logged');
  const wellnessColor = $derived(fallbackRating ? (heatmapColors[fallbackRating.wellness_score - 1] ?? 'var(--text-muted)') : 'var(--text-muted)');
  const wellnessSub = $derived(
    todayRating
      ? 'Today, based on your last entry'
      : fallbackRating
        ? `As of ${fallbackRating.log_date}`
        : 'Log today to see your wellness score'
  );

  const upcoming = upcomingCare(3);
  function fmtMonth(iso: string): string { return new Date(iso + 'T00:00').toLocaleDateString(undefined, { month: 'short' }).toUpperCase(); }
  function fmtDay(iso: string): string { return String(new Date(iso + 'T00:00').getDate()).padStart(2, '0'); }
</script>

<div class="dashboard">
  <div class="header">
    <div class="date-line">{fmtLongDate(today)}</div>
    <h1 class="greeting">{greeting(today)}</h1>
  </div>

  <div class="cards">
    <section class="card wellness-card">
      <header class="card-header">
        <span class="eyebrow">WELLNESS</span>
        <button class="link" onclick={() => onNavigate('daily-rating')}>Open daily log →</button>
      </header>
      <div class="wellness-word" style="color: {wellnessColor}">{wellnessLabel}</div>
      <div class="wellness-sub">{wellnessSub}</div>
      <div class="last7-label">LAST 7 DAYS</div>
      <div class="last7">
        {#each last7 as day}
          <div class="last7-col">
            <div class="last7-letter">{day.label}</div>
            <div class="last7-cell" style="background: {day.rating && heatmapColors[day.rating.wellness_score - 1] ? heatmapColors[day.rating.wellness_score - 1] : 'var(--border)'}"></div>
          </div>
        {/each}
      </div>
    </section>

    <section class="card diagnoses-card">
      <header class="card-header">
        <span class="eyebrow">ACTIVE DIAGNOSES</span>
        <button class="link" onclick={() => onNavigate('care-team')}>Care team →</button>
      </header>
      {#if diagnoses.length === 0}
        <div class="empty">No active diagnoses.</div>
      {:else}
        {#each diagnoses as d}
          <div class="dx-row">
            <div class="dx-main">
              <div class="dx-name">
                {d.name}
                {#if d.short_name}<span class="dx-code">{d.short_name}</span>{/if}
              </div>
              <div class="dx-sub">
                {#if d.onset_date}Since {d.onset_date}{/if}
                {#if d.source} · {d.source}{/if}
              </div>
            </div>
            {#if d.chronic}<span class="pill-tag">CHRONIC</span>{/if}
          </div>
        {/each}
      {/if}
    </section>

    <section class="card upcoming-card">
      <header class="card-header">
        <span class="eyebrow">UPCOMING CARE</span>
        <button class="link" onclick={() => onNavigate('care-team')}>See all →</button>
      </header>
      {#each upcoming as a}
        <div class="up-row">
          <div class="date-tile">
            <div class="date-tile-mo">{fmtMonth(a.date)}</div>
            <div class="date-tile-day">{fmtDay(a.date)}</div>
          </div>
          <div class="up-body">
            <div class="up-title">{a.title}</div>
            <div class="up-sub">{a.location}</div>
          </div>
        </div>
      {/each}
    </section>
  </div>

  <section class="card attention-card">
    <header class="card-header">
      <span class="eyebrow">NEEDS ATTENTION <span class="attention-count">{abnormals.length} abnormal values</span></span>
      <button class="link" onclick={() => onNavigate('lab-results')}>View all labs →</button>
    </header>
    {#each abnormals as r}
      <div class="attn-row">
        <div class="attn-name">{r.test_name}</div>
        <div class="attn-meta">
          <span class="attn-date">{r.test_date}</span>
          <span class="attn-value">{r.value ?? r.text_value}</span>
          <span class="attn-unit">{r.unit}</span>
          <span class="flag-pill" data-flag={r.flag}>{r.flag}</span>
        </div>
      </div>
    {/each}
  </section>
</div>

<style>
  .dashboard { max-width: 1160px; margin: 0 auto; padding: 24px 8px; }
  .header { margin-bottom: 24px; }
  .date-line { color: var(--text-muted); font-size: 13px; }
  .greeting { font-family: var(--font-heading); font-size: 32px; font-weight: 900; margin-top: 4px; }

  .cards { display: grid; grid-template-columns: repeat(3, 1fr); gap: 16px; margin-bottom: 16px; }
  @media (max-width: 1000px) { .cards { grid-template-columns: 1fr; } }

  .card {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 12px;
    padding: 18px 20px;
  }

  .card-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px; }
  .eyebrow {
    font-size: 11px; font-weight: 700; letter-spacing: 0.08em;
    color: var(--text-muted); text-transform: uppercase;
  }
  .link {
    background: none; border: none; color: var(--primary);
    font-size: 12px; font-weight: 600; cursor: pointer; padding: 0;
  }

  .wellness-word { font-family: var(--font-heading); font-size: 44px; font-weight: 900; line-height: 1; margin: 8px 0 4px; }
  .wellness-sub { color: var(--text-muted); font-size: 13px; margin-bottom: 20px; }
  .last7-label { font-size: 10px; letter-spacing: 0.08em; color: var(--text-muted); margin-bottom: 6px; }
  .last7 { display: grid; grid-template-columns: repeat(7, 1fr); gap: 6px; }
  .last7-col { display: flex; flex-direction: column; align-items: center; gap: 4px; }
  .last7-letter { font-size: 11px; color: var(--text-muted); }
  .last7-cell { width: 100%; aspect-ratio: 1; border-radius: 6px; }

  .dx-row { display: flex; justify-content: space-between; align-items: flex-start; padding: 8px 0; gap: 8px; }
  .dx-name { font-weight: 600; }
  .dx-code { font-weight: 500; color: var(--text-muted); margin-left: 6px; font-size: 12px; }
  .dx-sub { color: var(--text-muted); font-size: 12px; margin-top: 2px; }
  .empty { color: var(--text-muted); font-size: 13px; padding: 8px 0; }

  .pill-tag {
    background: color-mix(in oklab, var(--primary) 12%, transparent);
    color: var(--primary);
    font-size: 10px; font-weight: 700; letter-spacing: 0.06em;
    padding: 3px 8px; border-radius: 999px;
  }

  .up-row { display: flex; align-items: center; gap: 12px; padding: 8px 0; }
  .date-tile {
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 4px 10px;
    text-align: center;
    min-width: 46px;
  }
  .date-tile-mo { font-size: 10px; color: var(--text-muted); letter-spacing: 0.06em; }
  .date-tile-day { font-family: var(--font-heading); font-weight: 700; font-size: 16px; }
  .up-title { font-weight: 600; }
  .up-sub { color: var(--text-muted); font-size: 12px; }

  .attention-count { color: var(--accent-bad); margin-left: 6px; }
  .attn-row {
    display: flex; justify-content: space-between; align-items: center;
    padding: 10px 0; border-top: 1px solid var(--border);
  }
  .attn-row:first-of-type { border-top: none; }
  .attn-name { font-weight: 600; }
  .attn-meta { display: flex; align-items: center; gap: 12px; color: var(--text-muted); font-size: 12px; }
  .attn-value { font-family: var(--font-mono); color: var(--text); font-weight: 600; }
  .flag-pill {
    display: inline-flex; align-items: center; justify-content: center;
    width: 22px; height: 22px; border-radius: 6px;
    font-size: 11px; font-weight: 700; color: white;
  }
  .flag-pill[data-flag="H"], .flag-pill[data-flag="HH"] { background: var(--accent-high); }
  .flag-pill[data-flag="L"], .flag-pill[data-flag="LL"] { background: var(--accent-low); }
</style>
```

- [ ] **Step 3: Verify required db helpers exist**

Run:

```bash
grep -n "listDiagnoses\|getRatingsRange\|getAbnormalLabs\|getHeatmapConfig" src/lib/db.ts
```

Expected: all four functions present. If `getRatingsRange` does not exist under that name, check for `listRatings`/`getDailyRatings`/similar and use the actual name in the Dashboard import.

- [ ] **Step 4: Type check + visual verification**

Run: `bun run check`
Expected: zero errors. If missing helpers, add tiny wrappers in `src/lib/db.ts` calling the corresponding Tauri command (see existing patterns).

Run: `bun run tauri dev` → Dashboard shows greeting, three cards, needs attention list.

- [ ] **Step 5: Commit**

```bash
git add src/lib/views/Dashboard.svelte src/lib/db.ts
git commit -m "feat(dashboard): rewrite dashboard with wellness, diagnoses, upcoming, attention"
```

---

## Phase 5 — View restyles

### Task 11: Restyle Daily Log (smaller cells + themed panel)

**Files:**
- Modify: `src/lib/views/DailyRating.svelte`
- Modify: `src/lib/views/HeatmapGrid.svelte`
- Modify: `src/lib/views/DayEntryPanel.svelte`

- [ ] **Step 1: Shrink heatmap cell size**

Open `src/lib/views/HeatmapGrid.svelte`. Find the CSS rule sizing each cell (likely `.cell { width: XXpx; height: XXpx }` or a CSS grid `grid-auto-rows`). Change the cell dimension to `40px` (from whatever current, e.g. `60px`). Adjust `gap` proportionally (e.g. `4px`).

- [ ] **Step 2: Migrate hard-coded colors to tokens**

In `DailyRating.svelte`, `HeatmapGrid.svelte`, and `DayEntryPanel.svelte`, replace any hard-coded surface/border/text colors with the new tokens: `--surface`, `--bg`, `--border`, `--text`, `--text-muted`, `--primary`. Leave heatmap fill colors alone (they come from `heatmap_config`).

- [ ] **Step 3: Restyle the score selector**

In `DayEntryPanel.svelte`, ensure each score option renders as a **full-width colored bar** with the label rendered inside (white text). The colored bar background should use `heatmapColors[i]`. Example pattern:

```svelte
{#each config.labels as label, i}
  <button
    class="score-bar"
    class:selected={rating?.wellness_score === i + 1}
    style="background: {config.colors[i]}"
    onclick={() => setScore(i + 1)}
  >
    {label}
  </button>
{/each}
```

```css
.score-bar {
  display: block; width: 100%; padding: 12px 14px;
  border: 2px solid transparent; border-radius: 8px;
  color: white; font-weight: 600; text-align: left;
  margin-bottom: 6px; cursor: pointer;
}
.score-bar.selected { border-color: var(--primary); }
```

- [ ] **Step 4: Visual verification**

Run: `bun run tauri dev`
Navigate to Daily Log. Confirm heatmap cells are ~40px, panel matches mockup style (colored bars, white labels), colors respond to theme changes made in Settings.

- [ ] **Step 5: Commit**

```bash
git add src/lib/views/DailyRating.svelte src/lib/views/HeatmapGrid.svelte src/lib/views/DayEntryPanel.svelte
git commit -m "style(daily-log): shrink heatmap cells and restyle score selector"
```

---

### Task 12: Restyle Results as grouped-by-date cards

**Files:**
- Modify: `src/lib/views/LabResults.svelte`

- [ ] **Step 1: Inspect current structure**

Run: `head -80 src/lib/views/LabResults.svelte` — confirm the view already loads sessions with results. Note where each session renders.

- [ ] **Step 2: Restructure rendering**

Update the template so each `LabSessionWithResults` renders as a card:

```svelte
<div class="results-view">
  <div class="results-header">
    <h1>Lab Results</h1>
    <button class="primary" onclick={() => onNavigate('lab-entry')}>+ New Lab Entry</button>
  </div>

  {#each sessions as sr}
    <article class="session-card" class:expanded={expanded === sr.session.id}>
      <header class="session-header" onclick={() => toggle(sr.session.id)}>
        <span class="session-date">{sr.session.test_date}</span>
        <span class="session-lab">{sr.session.lab_name}</span>
        <span class="session-panel">{sr.session.notes}</span>
        <span class="flagged-pill" data-severity={flaggedSeverity(sr)}>{flaggedCount(sr)} flagged</span>
        <button class="edit-btn" onclick={(e) => { e.stopPropagation(); onNavigate('lab-entry', sr.session.id); }}>Edit</button>
      </header>

      {#if expanded === sr.session.id}
        <table class="results-table">
          <thead><tr><th>TEST</th><th>VALUE</th><th>UNIT</th><th>RANGE</th><th>FLAG</th></tr></thead>
          <tbody>
            {#each sr.results as r}
              <tr>
                <td class="test">{r.test_name}</td>
                <td class="value">{r.value ?? r.text_value}</td>
                <td class="unit">{r.unit}</td>
                <td class="range">
                  {#if r.ref_range_low !== null && r.ref_range_high !== null}
                    {r.ref_range_low}-{r.ref_range_high}
                  {/if}
                </td>
                <td><span class="flag-pill" data-flag={r.flag}>{r.flag}</span></td>
              </tr>
            {/each}
          </tbody>
        </table>
      {/if}
    </article>
  {/each}
</div>
```

Add script helpers:

```ts
let expanded: number | null = $state(null);
function toggle(id: number | null) { expanded = expanded === id ? null : id; }
function flaggedCount(sr: LabSessionWithResults): number {
  return sr.results.filter((r) => r.flag && r.flag !== 'N').length;
}
function flaggedSeverity(sr: LabSessionWithResults): 'high' | 'low' | 'mixed' {
  const flags = sr.results.filter((r) => r.flag && r.flag !== 'N').map((r) => r.flag);
  const hi = flags.some((f) => f === 'H' || f === 'HH');
  const lo = flags.some((f) => f === 'L' || f === 'LL');
  if (hi && lo) return 'mixed';
  return hi ? 'high' : 'low';
}
```

Auto-expand the first session on load:

```ts
$effect(() => { if (sessions.length && expanded === null) expanded = sessions[0].session.id; });
```

Add styles:

```css
.results-view { max-width: 1160px; margin: 0 auto; padding: 24px 8px; }
.results-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 20px; }
.results-header h1 { font-family: var(--font-heading); font-size: 28px; font-weight: 900; }

.session-card {
  background: var(--surface); border: 1px solid var(--border);
  border-radius: 12px; margin-bottom: 12px; overflow: hidden;
}
.session-header {
  display: flex; align-items: center; gap: 16px;
  padding: 18px 20px; cursor: pointer;
}
.session-date { font-weight: 700; white-space: nowrap; }
.session-lab { color: var(--text-muted); }
.session-panel { color: var(--text-muted); flex: 1; }
.flagged-pill {
  padding: 4px 10px; border-radius: 999px; font-size: 12px; font-weight: 600;
}
.flagged-pill[data-severity="high"] { background: color-mix(in oklab, var(--accent-high) 22%, transparent); color: var(--accent-high); }
.flagged-pill[data-severity="low"], .flagged-pill[data-severity="mixed"] {
  background: color-mix(in oklab, var(--accent-low) 15%, transparent); color: var(--accent-low);
}
.edit-btn {
  padding: 6px 14px; border-radius: 8px;
  border: 1px solid var(--border); background: var(--surface); color: var(--text);
}
.results-table { width: 100%; border-top: 1px solid var(--border); border-collapse: collapse; }
.results-table th {
  text-align: left; padding: 10px 20px; font-size: 11px;
  color: var(--text-muted); letter-spacing: 0.06em; font-weight: 600;
}
.results-table td { padding: 12px 20px; border-top: 1px solid var(--border); }
.results-table td.test { font-weight: 700; }
.results-table td.value { font-family: var(--font-mono); font-weight: 700; }
.results-table td.unit, .results-table td.range { color: var(--text-muted); }
.flag-pill {
  display: inline-flex; align-items: center; justify-content: center;
  width: 22px; height: 22px; border-radius: 6px;
  font-size: 11px; font-weight: 700; color: white;
}
.flag-pill[data-flag="H"], .flag-pill[data-flag="HH"] { background: var(--accent-high); }
.flag-pill[data-flag="L"], .flag-pill[data-flag="LL"] { background: var(--accent-low); }
```

- [ ] **Step 2: Type check + visual verification**

Run: `bun run check` — zero errors.
Run: `bun run tauri dev` — navigate to Results. Confirm grouped cards with flagged pill, click header expands/collapses.

- [ ] **Step 3: Commit**

```bash
git add src/lib/views/LabResults.svelte
git commit -m "style(results): grouped session cards with flagged pill and expand/collapse"
```

---

### Task 13: Restyle Trends view (theme tokens only)

**Files:**
- Modify: `src/lib/views/Trends.svelte`

- [ ] **Step 1: Replace hard-coded colors**

Open `src/lib/views/Trends.svelte`. Replace hard-coded hex values with tokens (`--surface`, `--border`, `--text`, `--text-muted`, `--primary`). For the chart itself (Chart.js), locate the `datasets` config and set `borderColor` and `pointBackgroundColor` to `getComputedStyle(document.documentElement).getPropertyValue('--primary').trim()`.

Example:

```ts
const primary = getComputedStyle(document.documentElement).getPropertyValue('--primary').trim();
const muted = getComputedStyle(document.documentElement).getPropertyValue('--text-muted').trim();
// ...pass to chart config
```

Wrap the chart container in `.trend-card`:

```css
.trend-card {
  background: var(--surface); border: 1px solid var(--border);
  border-radius: 12px; padding: 20px;
}
```

- [ ] **Step 2: Visual verification**

Run: `bun run tauri dev` → Trends view. Line color matches primary; page background matches theme.

- [ ] **Step 3: Commit**

```bash
git add src/lib/views/Trends.svelte
git commit -m "style(trends): apply theme tokens and card container"
```

---

### Task 14: Rename Artifacts → Documents view

**Files:**
- Rename: `src/lib/views/Artifacts.svelte` → `src/lib/views/Documents.svelte`
- Modify: `src/routes/+page.svelte` (imports + branch)

- [ ] **Step 1: Rename file**

Run:

```bash
git mv src/lib/views/Artifacts.svelte src/lib/views/Documents.svelte
```

- [ ] **Step 2: Update import and view name in Documents.svelte**

Open the renamed file. Change the top-level heading text to "Documents". Replace hard-coded colors with theme tokens (`--surface`, `--border`, `--text`, `--text-muted`).

If the current view has zero uploaded artifacts, ensure an empty state renders:

```svelte
{#if artifacts.length === 0}
  <div class="empty-state">
    <div class="empty-title">No documents yet</div>
    <div class="empty-body">Paste text or upload a file to save it here.</div>
  </div>
{/if}
```

```css
.empty-state {
  padding: 40px 20px; text-align: center;
  border: 1px dashed var(--border); border-radius: 12px;
  background: var(--surface);
}
.empty-title { font-family: var(--font-heading); font-weight: 700; font-size: 18px; }
.empty-body { color: var(--text-muted); margin-top: 6px; }
```

- [ ] **Step 3: Update root**

In `src/routes/+page.svelte`:
- Change `import Artifacts from '$lib/views/Artifacts.svelte';` to `import Documents from '$lib/views/Documents.svelte';`
- Change the `{:else if currentView === 'documents'}` branch to render `<Documents onNavigate={navigate} />`.

- [ ] **Step 4: Type check + verification**

Run: `bun run check` — zero errors.
Run: `bun run tauri dev` → Documents tab renders (existing artifacts still list; empty state if none).

- [ ] **Step 5: Commit**

```bash
git add src/lib/views/Documents.svelte src/routes/+page.svelte
git commit -m "refactor(documents): rename Artifacts view to Documents"
```

---

### Task 15: Build Care Team view (real diagnoses + placeholder columns)

**Files:**
- Rename: `src/lib/views/Diagnoses.svelte` → `src/lib/views/CareTeam.svelte`
- Modify: `src/routes/+page.svelte` (imports + branch)

- [ ] **Step 1: Rename**

Run:

```bash
git mv src/lib/views/Diagnoses.svelte src/lib/views/CareTeam.svelte
```

- [ ] **Step 2: Restructure**

Rewrite the file body to a two-column layout. Keep all existing diagnosis-CRUD logic in the script — only the surrounding markup changes. Wrap the existing diagnoses list in a left column; add a right column with two placeholder cards populated by `DEMO_APPOINTMENTS` and `DEMO_PROVIDERS`.

Template shell (adapt existing diagnoses list to fill the left column):

```svelte
<script lang="ts">
  // ...existing imports and state
  import { DEMO_APPOINTMENTS, DEMO_PROVIDERS } from '$lib/demo';

  function fmtMonth(iso: string): string { return new Date(iso + 'T00:00').toLocaleDateString(undefined, { month: 'short' }).toUpperCase(); }
  function fmtDay(iso: string): string { return String(new Date(iso + 'T00:00').getDate()).padStart(2, '0'); }
</script>

<div class="care-team-view">
  <h1>Care Team</h1>

  <div class="two-col">
    <section class="left-col">
      <div class="col-header">
        <span class="eyebrow">DIAGNOSES</span>
        <button onclick={openAdd}>+ Add diagnosis</button>
      </div>
      <!-- existing diagnoses list, wrapped per item in .dx-card matching mockup -->
      {#each diagnoses as d}
        <article class="dx-card">
          <header>
            <div class="dx-title">
              {d.name}
              {#if d.short_name}<span class="dx-code">{d.short_name}</span>{/if}
              {#if d.chronic}<span class="pill-tag">CHRONIC</span>{/if}
            </div>
            <div class="dx-actions">
              <button onclick={() => editDiagnosis(d)}>Edit</button>
              <button class="danger" onclick={() => deleteDiagnosis(d)}>Delete</button>
            </div>
          </header>
          <div class="dx-sub">
            {#if d.onset_date}Since {d.onset_date}{/if}
            {#if d.source} · {d.source}{/if}
          </div>
          {#if d.details}<p class="dx-details">{d.details}</p>{/if}
        </article>
      {/each}
    </section>

    <aside class="right-col">
      <section class="side-card">
        <div class="eyebrow">UPCOMING APPOINTMENTS</div>
        {#each DEMO_APPOINTMENTS as a}
          <div class="side-row">
            <div class="date-tile">
              <div class="date-tile-mo">{fmtMonth(a.date)}</div>
              <div class="date-tile-day">{fmtDay(a.date)}</div>
            </div>
            <div class="side-body">
              <div class="side-title">{a.title}</div>
              <div class="side-sub">{a.location}</div>
            </div>
          </div>
        {/each}
      </section>

      <section class="side-card">
        <div class="eyebrow">PROVIDERS</div>
        {#each DEMO_PROVIDERS as p}
          <div class="side-row">
            <div class="avatar">{p.initials}</div>
            <div class="side-body">
              <div class="side-title">{p.name}</div>
              <div class="side-sub">{p.specialty}</div>
            </div>
          </div>
        {/each}
      </section>
    </aside>
  </div>

  <!-- existing add/edit dialogs preserved -->
</div>

<style>
  .care-team-view { max-width: 1160px; margin: 0 auto; padding: 24px 8px; }
  .care-team-view h1 { font-family: var(--font-heading); font-size: 28px; font-weight: 900; margin-bottom: 20px; }
  .two-col { display: grid; grid-template-columns: 2fr 1fr; gap: 20px; }
  @media (max-width: 900px) { .two-col { grid-template-columns: 1fr; } }

  .col-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px; }
  .eyebrow {
    font-size: 11px; font-weight: 700; letter-spacing: 0.08em;
    color: var(--text-muted); text-transform: uppercase;
  }
  .dx-card {
    background: var(--surface); border: 1px solid var(--border);
    border-radius: 12px; padding: 18px 20px; margin-bottom: 12px;
  }
  .dx-card header { display: flex; justify-content: space-between; align-items: center; gap: 12px; }
  .dx-title { font-weight: 700; }
  .dx-code { font-weight: 500; color: var(--text-muted); font-size: 12px; margin-left: 6px; }
  .pill-tag {
    background: color-mix(in oklab, var(--primary) 12%, transparent);
    color: var(--primary); font-size: 10px; font-weight: 700;
    letter-spacing: 0.06em; padding: 3px 8px; border-radius: 999px; margin-left: 8px;
  }
  .dx-actions { display: flex; gap: 6px; }
  .dx-sub { color: var(--text-muted); font-size: 12px; margin: 6px 0; }
  .dx-details { margin-top: 8px; }

  .side-card {
    background: var(--surface); border: 1px solid var(--border);
    border-radius: 12px; padding: 18px 20px; margin-bottom: 16px;
  }
  .side-row { display: flex; align-items: center; gap: 12px; padding: 10px 0; }
  .side-row + .side-row { border-top: 1px solid var(--border); }
  .date-tile {
    background: var(--bg); border: 1px solid var(--border);
    border-radius: 8px; padding: 4px 10px; text-align: center; min-width: 46px;
  }
  .date-tile-mo { font-size: 10px; color: var(--text-muted); letter-spacing: 0.06em; }
  .date-tile-day { font-family: var(--font-heading); font-weight: 700; font-size: 16px; }
  .avatar {
    width: 40px; height: 40px; border-radius: 50%;
    background: var(--primary); color: var(--primary-contrast);
    display: flex; align-items: center; justify-content: center;
    font-weight: 700; font-size: 13px;
  }
  .side-title { font-weight: 600; }
  .side-sub { color: var(--text-muted); font-size: 12px; }
</style>
```

Preserve the file's existing state, event handlers (`openAdd`, `editDiagnosis`, `deleteDiagnosis`), and dialogs — only the shell around them changes.

- [ ] **Step 3: Update root**

In `src/routes/+page.svelte`:
- `import Diagnoses` → `import CareTeam from '$lib/views/CareTeam.svelte';`
- The `care-team` branch renders `<CareTeam />`.

- [ ] **Step 4: Type check + verification**

Run: `bun run check` — zero errors.
Run: `bun run tauri dev` → Care Team shows two columns; diagnosis CRUD still works.

- [ ] **Step 5: Commit**

```bash
git add src/lib/views/CareTeam.svelte src/routes/+page.svelte
git commit -m "feat(care-team): rename Diagnoses view, add appointments/providers placeholders"
```

---

## Phase 6 — Cleanup

### Task 16: Remove Chat surface from nav settings

**Files:**
- Modify: `src/lib/views/Settings.svelte`

- [ ] **Step 1: Remove chat toggle**

Locate any UI in `Settings.svelte` that toggles `chat_enabled` (search for `chat_enabled`). Remove the checkbox/label. Do NOT delete the backend command or table.

Run: `grep -n "chat" src/lib/views/Settings.svelte`
Expected after edit: no references remain.

- [ ] **Step 2: Verification**

Run: `bun run tauri dev` → Settings shows Theme + Export only (plus any other existing sections).

- [ ] **Step 3: Commit**

```bash
git add src/lib/views/Settings.svelte
git commit -m "chore(settings): hide chat toggle (feature removed from nav)"
```

---

### Task 17: Full manual verification pass

**No file changes.**

- [ ] **Step 1: Run type check and tests**

Run: `bun run check && bun run test:run`
Expected: zero type errors; all tests pass (including new `theme.test.ts`, `demo.test.ts`).

- [ ] **Step 2: End-to-end manual walk-through**

Run: `bun run tauri dev`

Verify for each nav item:

- **Dashboard** — greeting + date + three cards + Needs Attention list render; links navigate correctly; wellness word colored via heatmap ramp.
- **Daily Log** — smaller heatmap cells; day panel score bars styled; theme colors apply.
- **Results** — session cards with flagged pill; expand/collapse works; `+ New Lab Entry` opens Lab Entry.
- **Trends** — chart renders in themed card.
- **Documents** — renders list or empty state; "Documents" heading.
- **Care Team** — two columns; diagnoses CRUD works; appointments + providers placeholders show demo data.
- **Settings gear** — Theme presets swap live; individual color pickers update immediately and persist across app restart; Export produces a file.
- **`?` icon** — Glossary opens.
- **`+ Log today`** — navigates to Daily Log.

- [ ] **Step 3: Commit anything incidental**

If small fixes surfaced during verification, commit them individually with descriptive messages. No blanket "cleanup" commit.

---

## Self-review notes

- **Spec coverage:** Branding (T6), navigation (T5, T6), Dashboard (T9, T10), Daily Log restyle (T11), Results restyle (T12), Trends restyle (T13), Documents rename (T14), Care Team placeholders (T15), theme tokens + persistence (T1–T3, T7), fonts (T4), Chat hidden (T6, T16), Export moved (T8). All covered.
- **Placeholders in plan:** none.
- **Type consistency:** `View` union edits in T5 are propagated in T6, T10, T14, T15. `ThemeSetting` / `TokenKey` shapes stable across T1, T2, T7. `applyTheme` signature (`HTMLElement`, `ThemeSetting`) consistent T2, T3, T7. `DEMO_APPOINTMENTS` / `DEMO_PROVIDERS` types stable T9, T15.
- **DB helpers assumption:** T10 assumes `listDiagnoses`, `getRatingsRange`, `getAbnormalLabs`, `getHeatmapConfig` exist in `src/lib/db.ts`. T10 Step 3 has an explicit fallback if any are named differently — engineer adds a wrapper or renames the import to the actual name.
