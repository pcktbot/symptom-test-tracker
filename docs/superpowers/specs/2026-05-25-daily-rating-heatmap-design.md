# Daily Rating Heatmap — Design Spec

**Date:** 2026-05-25
**Branch:** rate-each-day

## Overview

Replace the `SymptomEntry` checkbox form with a minimal daily rating view. The primary interaction is: pick a color for today, optionally tag symptoms. The full-year heatmap is always visible. Historical years are navigable.

## Motivation

The existing symptom log form is overwhelming for daily use. This simplifies the core daily check-in to a single color click while preserving the ability to annotate with free-form tags.

---

## Layout

Three regions rendered in a single `DailyRating` view:

**1. Header bar**
- Left/right arrows to navigate years; current year label centered.
- Gear icon (right) toggles the config aside.

**2. Heatmap grid** (`HeatmapGrid.svelte`)
- Always visible. 31 rows (days 1–31) × 12 columns (Jan–Dec).
- Day numbers along the left; month names angled across the top.
- Cells colored by the stored score for that date using the active config palette.
- Invalid dates (e.g., Feb 30) rendered as non-interactive, visually muted.
- Today's cell has a subtle highlight ring.
- Clicking any valid cell selects it.

**3. Entry panel** (`DayEntryPanel.svelte`)
- Appears below the grid when a cell is selected; hidden otherwise.
- Contains: a row of colored step buttons (score selector) + a tag input.
- Clicking a score button saves immediately — no submit button.
- Clicking outside the panel or pressing Escape deselects.

**Config aside** (`HeatmapConfig.svelte`)
- Slides in from the right when toggled.
- One row per step: color swatch (native `<input type="color">`), label text field.
- Step count controlled by a +/− control (range: 2–10).
- Warning banner shown when step count differs from saved value: "Changing the number of steps will remap all existing scores proportionally."
- Changes saved on close.

---

## Data Model

### Schema change — `daily_summaries`

Add one column:

```sql
ALTER TABLE daily_summaries ADD COLUMN tags TEXT NOT NULL DEFAULT '';
```

`tags` stores a JSON array of strings, e.g. `["fatigue","brain fog"]`. Empty default is `'[]'` semantically but stored as `''` (treated as empty array in application code).

`wellness_score` remains an integer, **1-based** (1 = worst, N = best for an N-step scale). This matches existing data (1–5) — no migration needed. The UI maps score to color/label via `colors[score - 1]`.

### Settings key — `heatmap_config`

Stored in the existing `settings` table under key `heatmap_config`. Value is a JSON string:

```json
{
  "steps": 5,
  "labels": ["Worst", "Poor", "Okay", "Good", "Best"],
  "colors": ["#c0392b", "#e67e22", "#f1c40f", "#27ae60", "#1e8449"]
}
```

Index 0 = label/color for score 1 (worst). Default is 5-step stoplight palette above.

### Existing tables

`symptom_logs` and `symptoms` are left intact. Historical data remains available in Trends and exports. No new rows are written to `symptom_logs` going forward.

---

## Tauri Commands

| Command | Signature | Notes |
|---|---|---|
| `get_daily_ratings` | `(year: i32) → Vec<DailyRating>` | All `daily_summaries` rows for the year |
| `save_daily_rating` | `(date, score, tags) → ()` | Upsert into `daily_summaries` |
| `get_all_tags` | `() → Vec<String>` | Distinct tags across all rows, for autocomplete |
| `get_heatmap_config` | `() → HeatmapConfig` | Reads `settings` key; returns default if missing |
| `save_heatmap_config` | `(config: HeatmapConfig) → ()` | Writes `settings` key |
| `remap_scores` | `(old_steps: i32, new_steps: i32) → ()` | Proportional remap: `round((score - 1) / (old_steps - 1) * (new_steps - 1)) + 1`, clamped to `[1, new_steps]`. Runs in same transaction as `save_heatmap_config` when step count changes. |

`DailyRating` type:
```typescript
interface DailyRating {
  log_date: string;   // YYYY-MM-DD
  score: number;      // 1-based integer (1 = worst, N = best)
  tags: string[];
}
```

---

## Frontend Components

| File | Responsibility |
|---|---|
| `src/lib/views/DailyRating.svelte` | Top-level view. Owns: selected year, selected date, config-open state. Fetches ratings on mount and year change. |
| `src/lib/views/HeatmapGrid.svelte` | Pure display. Props: ratings map, config, selected date. Emits `select(date)`. |
| `src/lib/views/DayEntryPanel.svelte` | Entry UI. Props: selected date, existing score + tags. Emits `save(score, tags)`. Auto-saves on score click. |
| `src/lib/views/HeatmapConfig.svelte` | Config aside. Reads/writes heatmap config via Tauri. Triggers remap when step count changes. |
| `src/lib/TagInput.svelte` | Reusable tag input. Props: current tags, suggestions. Enter or comma confirms; backspace removes last. |
| `src/lib/heatmap.ts` | Pure utilities: score→color lookup, step remapping math, tag serialization/deserialization. |

`SymptomEntry.svelte` is deleted. `SymptomEditor.svelte` is kept (used for managing symptom definitions independently).

Nav entry: the existing "Log" nav item under Symptoms is updated to point to `daily-rating` instead of `symptoms`.

---

## Error Handling

- All Tauri command errors surface as an inline error message in `DailyRating.svelte`.
- `remap_scores` and `save_heatmap_config` are wrapped in a single SQLite transaction — if the remap fails, the config is not persisted.
- Invalid date cells in the grid are non-interactive and never trigger a save.

---

## Testing

`src/lib/heatmap.ts` is unit-tested with vitest:
- Score-to-color mapping for all steps
- Proportional remap math (edge cases: step count 1→5, 5→1, 5→3)
- Tag parse/serialize round-trip

Tauri commands and UI flows are verified manually via the running app, consistent with the rest of the codebase.
