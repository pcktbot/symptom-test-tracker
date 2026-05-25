# Navigation Redesign

**Date:** 2026-05-23  
**Branch:** adjusting-test-labels  
**Status:** Approved

## Overview

Reduce top-level nav clutter by demoting configure/manage views to slide-in panels, surfacing Export as an action within Artifacts, restructuring the toolbar layout, adding a resizable chat sidebar, persisting text size and window dimensions, and binding Cmd+/- for font size control.

## Toolbar Layout

- Section labels move **above** their button groups, left-aligned (same small-caps/muted style as current inline labels).
- Settings (gear icon) and About (?) buttons move from `toolbar-left` to the **right end of the nav**, in that order, before the chat button.
- Chat button becomes **icon-only** (speech bubble SVG), always the last item in the nav. No text label.
- The `toolbar-left` cluster retains only the app title.

## Configure Panels

### Lab Configure
- `LabManage.svelte` gains an `onClose` prop (matching Glossary's interface).
- Removed from the `View` union type and the view-switcher block in `+page.svelte`.
- `+page.svelte` adds `labConfigOpen: boolean` state.
- A callback prop `openLabConfig` is threaded into `LabResults` and `Trends`. Each view shows a **"Configure" button** (gear icon + text) in its view header.
- Rendered as an inline-or-overlay panel using the existing `bodyWidth` / `CONTENT_MAX + PANEL_WIDTH` threshold logic.

### Symptom Configure
- `SymptomEditor.svelte` gains an `onClose` prop.
- Removed from the `View` union type and view-switcher.
- `+page.svelte` adds `symptomConfigOpen: boolean` state.
- `SymptomEntry` receives an `openSymptomConfig` callback and shows a **"Configure"** button in its header.
- Same inline/overlay panel logic.

### Panel width
- Both configure panels use `width: 400px` (between Glossary's 340px and the chat panel's variable width).

## Export Panel

- `Export.svelte` gains an `onClose` prop.
- Removed from the `View` union type, `navGroups`, and view-switcher.
- `+page.svelte` adds `exportOpen: boolean` state.
- `Artifacts.svelte` receives an `openExport` callback and shows an **"Export" button** alongside its existing header controls.
- Rendered as an inline/overlay panel (same pattern, `width: 360px`).
- The `Data` nav group becomes just `Artifacts`.

## Chat Sidebar — Drag-to-Resize

- A **drag handle** (4px wide, full-height, left edge of the chat panel) responds to `mousedown` → `mousemove` → `mouseup` to resize the panel.
- Width clamped: **min 240px, max 600px**.
- `chatWidth` state in `+page.svelte` (replaces the hardcoded `320px`).
- Width persisted to the `settings` table under key `chat_width` on drag end. Loaded on startup via `getSetting('chat_width')`.

## Text Size

- New setting `font_size` in the `settings` table. Values: `sm` | `md` | `lg`. Default: `md`.
- Applied as `data-font-size="md"` on `<body>`. CSS in `app.css` maps this to a `--font-scale` custom property:
  - `sm`: 0.9
  - `md`: 1.0 (base)
  - `lg`: 1.1
- All `font-size` values that should scale use `calc(Xpx * var(--font-scale))`, or a base `font-size` override on `:root`.
- A three-option control (Small / Medium / Large) is added to the **Settings modal**.
- **Keybindings** registered in `+page.svelte` via a `keydown` listener on `window`:
  - `Cmd+=` or `Cmd++` → increase size
  - `Cmd+-` → decrease size
  - `Cmd+0` → reset to `md`

## Window Dimensions

- On startup, `+page.svelte` reads `window_width`, `window_height`, `window_x`, `window_y` from the `settings` table and calls the Tauri `set_window_size` / `set_window_position` commands.
- A `ResizeObserver` or Tauri window event listener saves updated dimensions to settings on resize/move (debounced ~500ms).
- Rust side: two new Tauri commands `set_window_size(width, height)` and `set_window_position(x, y)` in a new `commands/window.rs`, or added to `commands/settings.rs`.
- Window is shown **after** position/size are restored to avoid flicker (use `visible: false` in `tauri.conf.json` + call `window.show()` after restore).

## View Type Cleanup

Remove from `View` union: `lab-manage`, `symptom-editor`, `export`.  
Remove from `navGroups`: Manage (Labs), Manage (Symptoms), Export (Data).  
Remove corresponding `{:else if}` blocks from the view-switcher.

## Files Affected

| File | Change |
|------|--------|
| `src/routes/+page.svelte` | Toolbar restructure, panel state, keybindings, window restore, font size |
| `src/lib/types.ts` | Remove `lab-manage`, `symptom-editor`, `export` from `View` |
| `src/lib/views/LabManage.svelte` | Add `onClose` prop |
| `src/lib/views/SymptomEditor.svelte` | Add `onClose` prop |
| `src/lib/views/Export.svelte` | Add `onClose` prop |
| `src/lib/views/LabResults.svelte` | Add Configure button, accept `openLabConfig` prop |
| `src/lib/views/Trends.svelte` | Add Configure button, accept `openLabConfig` prop |
| `src/lib/views/SymptomEntry.svelte` | Add Configure button, accept `openSymptomConfig` prop |
| `src/lib/views/Artifacts.svelte` | Add Export button, accept `openExport` prop |
| `src/lib/views/Settings.svelte` | Add font size control |
| `src/lib/db.ts` | No changes needed (uses existing `getSetting`/`setSetting`) |
| `src/app.css` | Add `--font-scale` variable and `data-font-size` rules |
| `src-tauri/src/commands/settings.rs` | Add window size/position get+set commands |
| `src-tauri/src/lib.rs` | Register new window commands |
