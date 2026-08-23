# Baseline: Theming & View Reorganization — Design Spec

**Date:** 2026-07-30
**Branch:** `theming`
**Reference mockup:** `~/Desktop/Symptom Tracker (standalone).html` (Clarion mockup — rebranded to Baseline)

## Goal

Rebrand the app to **Baseline**, restructure navigation, redesign the Dashboard, restyle existing views to match the mockup, and introduce a user-customizable theme system (colors + presets, bundled fonts) that obviates a separate dark mode. New entities in the mockup (Appointments, Providers, Documents) ship as visual placeholders — real CRUD is deferred.

## Non-goals

- No schema, command, or CRUD for Appointments or Providers (seeded demo data only).
- No real Documents functionality (visual restyle of existing Artifacts view).
- No Chat surface (hidden from nav; code + `chat_messages` table retained).
- No MCP server changes.
- No dark mode (superseded by user-editable theme tokens).
- No changes to the heatmap color model (stays per-step editable via HeatmapConfig).
- No data migrations beyond a single new `theme` key in the existing `settings` table.

## Branding

- App name displayed as **Baseline** across UI (window title, top bar, welcome).
- No logo asset. Header uses a filled dark-primary square tile (empty) + "Baseline" wordmark, matching the mockup.
- Package/binary names in `Cargo.toml` unchanged.

## Navigation

Sidebar removed. New top bar:

- **Left:** primary-color square tile + "Baseline" wordmark.
- **Center:** pill-group nav — `Dashboard | Daily Log | Results | Trends | Documents | Care Team`. Active item shown with filled primary background.
- **Right:** `+ Log today` primary button (navigates to Daily Log with today's day panel open) · `?` icon (opens Glossary) · gear icon (opens Settings).

### View mapping

| Nav item     | Backed by                     | Change                                                                       |
| ------------ | ----------------------------- | ---------------------------------------------------------------------------- |
| Dashboard    | `Dashboard.svelte`            | Full rewrite (see Dashboard section)                                         |
| Daily Log    | `DailyRating.svelte`          | Restyle; reduce heatmap cell size; day-panel restyle                         |
| Results      | `LabResults.svelte`           | Restructure as grouped-by-date cards with flagged-count pill                 |
| Trends       | `Trends.svelte`               | Themed restyle only                                                          |
| Documents    | `Artifacts.svelte` (renamed)  | Themed restyle; empty-state visual                                           |
| Care Team    | `Diagnoses.svelte` (expanded) | Two-column: real Diagnoses + placeholder Appointments/Providers seeded demo  |

### Hidden from nav (still reachable)

- **Settings** — gear icon top-right. Includes new Theme panel + Export section + existing MCP toggles.
- **Glossary** — `?` icon.
- **LabEntry / LabManage** — from Results (`+ New Lab Entry` and per-row Edit).
- **HeatmapConfig / SymptomEditor** — from within Daily Log (existing entry points).
- **Welcome** — first-run only, unchanged.
- **Chat** — hidden entirely; `Chat.svelte`, `commands/chat.rs`, and `chat_messages` table retained (no cost to leaving as dead code).

## Theme system

### CSS variables (defined in `src/app.css` or a new `theme.css`)

```
--bg, --surface, --border
--text, --text-muted
--primary, --primary-contrast
--accent-high, --accent-low, --accent-good, --accent-bad
--radius-sm, --radius-md, --radius-lg
--font-sans, --font-mono
```

All views migrate hard-coded colors to these variables. The heatmap colors are unaffected (they remain in `heatmap_config`).

### Persistence

- New key `theme` in the existing `settings` table, value is JSON: `{ "preset": string, "overrides": { "<token>": "<value>" } }`.
- On app start, frontend reads `theme` setting and applies token values to `:root` via inline style.
- Changes in Settings apply immediately (debounced write, no explicit Save button).

### Presets (built-in)

- **Baseline Warm** (default) — cream bg, deep teal primary, warm accents (matches mockup).
- **Cool Slate** — light gray bg, indigo primary, cool accents.
- **High Contrast** — near-white bg, black text, saturated primary.

Selecting a preset resets overrides to the preset's values.

### Theme panel (in Settings)

- Preset dropdown.
- Grouped color pickers:
  - **Surfaces:** bg, surface, border
  - **Text:** text, muted
  - **Brand:** primary
  - **Accents:** high, low, good, bad
- "Reset to preset defaults" button.
- Live preview (changes apply immediately to app).

### Fonts

- Bundle `@fontsource/lato`, `@fontsource/ibm-plex-sans`, `@fontsource/ibm-plex-mono` (npm packages, self-hosted — no CDN, no external font loading).
- `--font-sans: 'IBM Plex Sans', system-ui, sans-serif;` — used for body text.
- Headings use `Lato`.
- `--font-mono: 'IBM Plex Mono', ui-monospace, monospace;` — used for values in labs.

## Dashboard redesign

Layout (top-down):

1. **Header** — muted date line ("Thursday, July 30, 2026") + big greeting ("Good morning/afternoon/evening" based on local time).
2. **Three cards** (equal-width row, wraps on narrow):
   - **Wellness** — big colored word for today's score (Worst/Rough/Poor/Mid/Good Enough/Healthy) colored via the heatmap ramp. Sub-label "Today, based on your last entry" — if today has no rating, fall back to the most recent rated day and show "As of {date}" as sub-label. "LAST 7 DAYS" mini strip of 7 heatmap cells with weekday letters (M T W T F S S). Header link "Open daily log →".
   - **Active Diagnoses** — list of diagnoses (name, code, "Since {date} · {specialty}", CHRONIC pill). Header link "Care team →".
   - **Upcoming Care** — three date-tile rows (`MON` `DD` + title + subtitle). **Seeded demo data** (hard-coded array in the component or a util) — no schema.
3. **Needs Attention** wide card — top N (5) abnormal labs from existing `get_abnormal_labs` command. Row: test name (left) / date + value + unit + flag pill (right). Header shows "NEEDS ATTENTION {N} abnormal values" + "View all labs →".

## Other view restyles

- **Daily Log**: shrink heatmap cell size (~40px). Restyle day-side panel: score selector as full-width colored bars with the label rendered inside the bar (per mockup). Tag input restyled to match.
- **Results**: card-per-session grouped view. Header row: date · location · panel-title · flagged-count pill · Edit button. Table beneath: Test | Value | Unit | Range | Flag. `+ New Lab Entry` button top-right of the view header.
- **Trends**: chart in themed card; muted axis labels; primary color for series stroke.
- **Documents** (Artifacts, renamed): themed cards + empty-state visual (icon + prompt).
- **Care Team**: two-column. Left: Diagnoses (existing model, restyled) + `+ Add diagnosis`. Right: Upcoming Appointments card + Providers card, both filled with **seeded demo entries** and non-functional Edit/Delete controls (read-only).
- **Settings**: reorganized into sections — **Theme** (new), **Export** (moved in from nav), **MCP** (existing toggles), **Danger Zone** (existing if any).

## Implementation surface (summary)

- Frontend rewrite of `+page.svelte` navigation shell.
- Rewrite: `Dashboard.svelte`.
- Restyle: `DailyRating.svelte`, `DayEntryPanel.svelte`, `HeatmapGrid.svelte`, `LabResults.svelte`, `Trends.svelte`, `Artifacts.svelte`, `Diagnoses.svelte`, `Settings.svelte`, `Welcome.svelte`.
- New: `theme.ts` (preset defs, token application), Theme panel UI inside Settings.
- New: seeded demo data utilities for Appointments, Providers, Upcoming Care.
- CSS: introduce theme tokens; migrate hard-coded colors.
- Fonts: install and import `@fontsource/*` packages.
- Rust: no command changes; `theme` uses existing `settings` get/set commands.
