# Daily Rating Heatmap — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the SymptomEntry checkbox form with a heatmap calendar (31 rows × 12 columns) where each cell represents a day, colored by a 1–5 wellness score, with a click-to-rate entry panel and optional free-form symptom tags.

**Architecture:** `daily_summaries` gets a `tags TEXT` column; a new `heatmap_config` settings key stores the configurable scale. Five new Tauri commands handle data access. Five new Svelte components compose the view: `HeatmapGrid` (display), `DayEntryPanel` (entry), `HeatmapConfig` (aside), `TagInput` (reusable), and `DailyRating` (top-level).

**Tech Stack:** Rust/rusqlite (serde_json already in Cargo.toml), Svelte 5 runes, TypeScript, vitest

---

## File Map

| Action | Path | Responsibility |
|---|---|---|
| Modify | `src-tauri/src/db.rs` | Add `tags` column migration |
| Create | `src-tauri/src/commands/daily_rating.rs` | 5 Tauri commands |
| Modify | `src-tauri/src/commands/mod.rs` | Register new module |
| Modify | `src-tauri/src/lib.rs` | Register 5 new commands |
| Modify | `src/lib/types.ts` | Add `DailyRating`, `HeatmapConfig` types; add `'daily-rating'` to `View` |
| Modify | `src/lib/db.ts` | Add 5 frontend invoke wrappers |
| Create | `src/lib/heatmap.ts` | Pure utilities: score→color, remap, tag parse/serialize |
| Create | `src/lib/heatmap.test.ts` | vitest unit tests for heatmap.ts |
| Create | `src/lib/TagInput.svelte` | Reusable tag input with autocomplete |
| Create | `src/lib/views/HeatmapGrid.svelte` | 31×12 grid, pure display |
| Create | `src/lib/views/DayEntryPanel.svelte` | Score buttons + tag input below grid |
| Create | `src/lib/views/HeatmapConfig.svelte` | Config aside: steps, labels, colors |
| Create | `src/lib/views/DailyRating.svelte` | Top-level view, wires everything |
| Modify | `src/routes/+page.svelte` | Swap `symptoms` → `daily-rating`, remove symptomConfig panel |
| Delete | `src/lib/views/SymptomEntry.svelte` | Replaced by DailyRating |

---

## Task 1: DB Schema Migration

**Files:**
- Modify: `src-tauri/src/db.rs`

- [ ] **Step 1: Add `tags` column migration after the main `execute_batch` call**

In `src-tauri/src/db.rs`, after line 129 (the closing `?;` of `execute_batch`), add:

```rust
        // Additive migration: tags column for daily_summaries
        conn.execute(
            "ALTER TABLE daily_summaries ADD COLUMN tags TEXT NOT NULL DEFAULT ''",
            [],
        )
        .or_else(|e| {
            if e.to_string().contains("duplicate column name") {
                Ok(0)
            } else {
                Err(e)
            }
        })?;
```

The full `migrate` function ending becomes:

```rust
            INSERT OR IGNORE INTO settings (key, value) VALUES ('anthropic_api_key', '');
            INSERT OR IGNORE INTO settings (key, value) VALUES ('chat_enabled', 'true');
            "
        )?;

        // Additive migration: tags column for daily_summaries
        conn.execute(
            "ALTER TABLE daily_summaries ADD COLUMN tags TEXT NOT NULL DEFAULT ''",
            [],
        )
        .or_else(|e| {
            if e.to_string().contains("duplicate column name") {
                Ok(0)
            } else {
                Err(e)
            }
        })?;

        // Seed default symptoms if table is empty
```

- [ ] **Step 2: Verify it compiles**

```bash
cd src-tauri && cargo check 2>&1 | tail -5
```

Expected: `Finished` with no errors.

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/db.rs
git commit -m "feat: add tags column to daily_summaries"
```

---

## Task 2: Create `daily_rating.rs` Commands

**Files:**
- Create: `src-tauri/src/commands/daily_rating.rs`

- [ ] **Step 1: Create the file with all structs and commands**

```rust
use crate::db::Database;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Serialize, Deserialize)]
pub struct DailyRating {
    pub log_date: String,
    pub wellness_score: i64,
    pub tags: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HeatmapConfig {
    pub steps: i64,
    pub labels: Vec<String>,
    pub colors: Vec<String>,
}

impl Default for HeatmapConfig {
    fn default() -> Self {
        HeatmapConfig {
            steps: 5,
            labels: vec![
                "Worst".to_string(),
                "Poor".to_string(),
                "Okay".to_string(),
                "Good".to_string(),
                "Best".to_string(),
            ],
            colors: vec![
                "#c0392b".to_string(),
                "#e67e22".to_string(),
                "#f1c40f".to_string(),
                "#27ae60".to_string(),
                "#1e8449".to_string(),
            ],
        }
    }
}

#[tauri::command]
pub fn get_daily_ratings(db: State<Database>, year: i32) -> Result<Vec<DailyRating>, String> {
    let conn = db.conn.lock().unwrap();
    let pattern = format!("{}%", year);
    let mut stmt = conn
        .prepare(
            "SELECT log_date, wellness_score, tags FROM daily_summaries
             WHERE log_date LIKE ?1
             ORDER BY log_date ASC",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(params![pattern], |row| {
            let tags_str: String = row.get(2)?;
            let tags: Vec<String> = if tags_str.is_empty() {
                vec![]
            } else {
                serde_json::from_str(&tags_str).unwrap_or_default()
            };
            Ok(DailyRating {
                log_date: row.get(0)?,
                wellness_score: row.get(1)?,
                tags,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_daily_rating(
    db: State<Database>,
    date: String,
    score: i64,
    tags: Vec<String>,
) -> Result<(), String> {
    let conn = db.conn.lock().unwrap();
    let tags_json = serde_json::to_string(&tags).map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO daily_summaries (log_date, wellness_score, tags)
         VALUES (?1, ?2, ?3)
         ON CONFLICT(log_date) DO UPDATE SET wellness_score = excluded.wellness_score,
                                             tags = excluded.tags",
        params![date, score, tags_json],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn get_all_tags(db: State<Database>) -> Result<Vec<String>, String> {
    let conn = db.conn.lock().unwrap();
    let mut stmt = conn
        .prepare("SELECT tags FROM daily_summaries WHERE tags != '' AND tags != '[]'")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(|e| e.to_string())?;
    let mut tag_set = std::collections::HashSet::new();
    for row in rows {
        let tags_str = row.map_err(|e| e.to_string())?;
        let tags: Vec<String> = serde_json::from_str(&tags_str).unwrap_or_default();
        for tag in tags {
            tag_set.insert(tag);
        }
    }
    let mut result: Vec<String> = tag_set.into_iter().collect();
    result.sort();
    Ok(result)
}

#[tauri::command]
pub fn get_heatmap_config(db: State<Database>) -> Result<HeatmapConfig, String> {
    let conn = db.conn.lock().unwrap();
    let result: Result<String, rusqlite::Error> = conn.query_row(
        "SELECT value FROM settings WHERE key = 'heatmap_config'",
        [],
        |row| row.get(0),
    );
    match result {
        Ok(json) => serde_json::from_str(&json).map_err(|e| e.to_string()),
        Err(_) => Ok(HeatmapConfig::default()),
    }
}

#[tauri::command]
pub fn save_heatmap_config(
    db: State<Database>,
    config: HeatmapConfig,
    old_steps: Option<i64>,
) -> Result<(), String> {
    let mut conn = db.conn.lock().unwrap();
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    if let Some(old) = old_steps {
        if old != config.steps && old > 1 && config.steps > 1 {
            tx.execute(
                "UPDATE daily_summaries SET wellness_score =
                 MIN(?2, MAX(1, CAST(ROUND(
                   (CAST(wellness_score AS REAL) - 1.0) / (CAST(?1 AS REAL) - 1.0)
                   * (CAST(?2 AS REAL) - 1.0) + 1.0
                 ) AS INTEGER)))",
                params![old, config.steps],
            )
            .map_err(|e| e.to_string())?;
        }
    }
    let json = serde_json::to_string(&config).map_err(|e| e.to_string())?;
    tx.execute(
        "INSERT INTO settings (key, value) VALUES ('heatmap_config', ?1)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![json],
    )
    .map_err(|e| e.to_string())?;
    tx.commit().map_err(|e| e.to_string())?;
    Ok(())
}
```

- [ ] **Step 2: Verify it compiles**

```bash
cd src-tauri && cargo check 2>&1 | tail -5
```

Expected: warnings only (unused), no errors. (The module isn't registered yet so `dead_code` warnings are expected.)

---

## Task 3: Register the New Module and Commands

**Files:**
- Modify: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Add module to `mod.rs`**

In `src-tauri/src/commands/mod.rs`, add:

```rust
pub mod daily_rating;
```

(Add it alongside the existing `pub mod symptoms;` etc.)

- [ ] **Step 2: Register commands in `lib.rs`**

In `src-tauri/src/lib.rs`, inside `tauri::generate_handler![...]`, add after `commands::symptoms::save_symptom_log,`:

```rust
            commands::daily_rating::get_daily_ratings,
            commands::daily_rating::save_daily_rating,
            commands::daily_rating::get_all_tags,
            commands::daily_rating::get_heatmap_config,
            commands::daily_rating::save_heatmap_config,
```

- [ ] **Step 3: Verify it compiles**

```bash
cd src-tauri && cargo check 2>&1 | tail -5
```

Expected: `Finished` with no errors.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/commands/daily_rating.rs src-tauri/src/commands/mod.rs src-tauri/src/lib.rs
git commit -m "feat: add daily_rating Tauri commands"
```

---

## Task 4: Frontend Types and DB Wrappers

**Files:**
- Modify: `src/lib/types.ts`
- Modify: `src/lib/db.ts`

- [ ] **Step 1: Add types to `types.ts`**

Add after the existing type definitions (e.g., after the last `export interface` or `export type`):

```typescript
export interface DailyRating {
  log_date: string;
  wellness_score: number; // 1-based: 1 = worst, N = best
  tags: string[];
}

export interface HeatmapConfig {
  steps: number;
  labels: string[];
  colors: string[];
}
```

Also update the `View` union type (currently line 87) — add `'daily-rating'` and remove `'symptoms'`:

```typescript
export type View = 'dashboard' | 'lab-results' | 'lab-entry' | 'trends' | 'daily-rating' | 'welcome' | 'artifacts' | 'diagnoses';
```

- [ ] **Step 2: Add invoke wrappers to `db.ts`**

First, add `DailyRating` and `HeatmapConfig` to the existing named import at the top of `src/lib/db.ts`:

```typescript
import type {
  // ... existing types ...
  DailyRating,
  HeatmapConfig,
} from './types';
```

Then add the following functions at the end of `src/lib/db.ts`:

```typescript
export function getDailyRatings(year: number): Promise<DailyRating[]> {
  return invoke('get_daily_ratings', { year });
}

export function saveDailyRating(date: string, score: number, tags: string[]): Promise<void> {
  return invoke('save_daily_rating', { date, score, tags });
}

export function getAllTags(): Promise<string[]> {
  return invoke('get_all_tags');
}

export function getHeatmapConfig(): Promise<HeatmapConfig> {
  return invoke('get_heatmap_config');
}

export function saveHeatmapConfig(config: HeatmapConfig, oldSteps: number | null): Promise<void> {
  return invoke('save_heatmap_config', { config, oldSteps });
}
```

Note: `invoke` is already imported at the top of `db.ts`.

- [ ] **Step 3: Type-check**

```bash
bun run check 2>&1 | tail -20
```

Expected: 0 errors (some warnings about unused SymptomEntry import are fine — that gets cleaned up in Task 11).

- [ ] **Step 4: Commit**

```bash
git add src/lib/types.ts src/lib/db.ts
git commit -m "feat: add DailyRating types and db wrappers"
```

---

## Task 5: `heatmap.ts` (TDD)

**Files:**
- Create: `src/lib/heatmap.test.ts`
- Create: `src/lib/heatmap.ts`

- [ ] **Step 1: Write the failing tests**

Create `src/lib/heatmap.test.ts`:

```typescript
import { describe, it, expect } from 'vitest';
import {
  scoreToColor,
  scoreToLabel,
  remapScore,
  parseTags,
  serializeTags,
  isValidDate,
  DEFAULT_CONFIG,
} from './heatmap';

describe('scoreToColor', () => {
  it('returns color for score 1 (worst)', () => {
    expect(scoreToColor(1, DEFAULT_CONFIG)).toBe('#c0392b');
  });
  it('returns color for score 5 (best)', () => {
    expect(scoreToColor(5, DEFAULT_CONFIG)).toBe('#1e8449');
  });
  it('returns empty string for score 0', () => {
    expect(scoreToColor(0, DEFAULT_CONFIG)).toBe('');
  });
  it('returns empty string for score above steps', () => {
    expect(scoreToColor(6, DEFAULT_CONFIG)).toBe('');
  });
});

describe('scoreToLabel', () => {
  it('returns label for score 1', () => {
    expect(scoreToLabel(1, DEFAULT_CONFIG)).toBe('Worst');
  });
  it('returns label for score 3', () => {
    expect(scoreToLabel(3, DEFAULT_CONFIG)).toBe('Okay');
  });
  it('returns empty string for out-of-range score', () => {
    expect(scoreToLabel(0, DEFAULT_CONFIG)).toBe('');
  });
});

describe('remapScore', () => {
  it('maps score 1 to 1 regardless of step counts', () => {
    expect(remapScore(1, 5, 3)).toBe(1);
  });
  it('maps max score to max score', () => {
    expect(remapScore(5, 5, 3)).toBe(3);
  });
  it('maps midpoint proportionally (5→3)', () => {
    expect(remapScore(3, 5, 3)).toBe(2);
  });
  it('maps 1-step to 5-step: min stays 1', () => {
    expect(remapScore(1, 3, 5)).toBe(1);
  });
  it('maps midpoint proportionally (3→5)', () => {
    expect(remapScore(2, 3, 5)).toBe(3);
  });
  it('maps max (3→5): becomes 5', () => {
    expect(remapScore(3, 3, 5)).toBe(5);
  });
  it('identity: same step count returns same score', () => {
    expect(remapScore(3, 5, 5)).toBe(3);
    expect(remapScore(1, 5, 5)).toBe(1);
  });
  it('clamps to [1, newSteps]', () => {
    expect(remapScore(5, 5, 2)).toBeGreaterThanOrEqual(1);
    expect(remapScore(5, 5, 2)).toBeLessThanOrEqual(2);
  });
});

describe('parseTags', () => {
  it('parses a JSON array string', () => {
    expect(parseTags('["fatigue","brain fog"]')).toEqual(['fatigue', 'brain fog']);
  });
  it('returns an array as-is', () => {
    expect(parseTags(['fatigue'])).toEqual(['fatigue']);
  });
  it('returns empty array for empty string', () => {
    expect(parseTags('')).toEqual([]);
  });
  it('returns empty array for empty JSON array', () => {
    expect(parseTags('[]')).toEqual([]);
  });
});

describe('serializeTags', () => {
  it('serializes to JSON string', () => {
    expect(serializeTags(['fatigue', 'brain fog'])).toBe('["fatigue","brain fog"]');
  });
  it('round-trips with parseTags', () => {
    const tags = ['fatigue', 'brain fog'];
    expect(parseTags(serializeTags(tags))).toEqual(tags);
  });
});

describe('isValidDate', () => {
  it('accepts Jan 31', () => {
    expect(isValidDate(2026, 1, 31)).toBe(true);
  });
  it('accepts Feb 28', () => {
    expect(isValidDate(2026, 2, 28)).toBe(true);
  });
  it('rejects Feb 30', () => {
    expect(isValidDate(2026, 2, 30)).toBe(false);
  });
  it('rejects April 31', () => {
    expect(isValidDate(2026, 4, 31)).toBe(false);
  });
  it('accepts Feb 29 on a leap year', () => {
    expect(isValidDate(2024, 2, 29)).toBe(true);
  });
  it('rejects Feb 29 on a non-leap year', () => {
    expect(isValidDate(2026, 2, 29)).toBe(false);
  });
});
```

- [ ] **Step 2: Run the tests to verify they fail**

```bash
bun run test:run 2>&1 | tail -15
```

Expected: multiple failures with "Cannot find module './heatmap'" or similar.

- [ ] **Step 3: Create `src/lib/heatmap.ts`**

```typescript
import type { HeatmapConfig } from './types';

export const DEFAULT_CONFIG: HeatmapConfig = {
  steps: 5,
  labels: ['Worst', 'Poor', 'Okay', 'Good', 'Best'],
  colors: ['#c0392b', '#e67e22', '#f1c40f', '#27ae60', '#1e8449'],
};

export function scoreToColor(score: number, config: HeatmapConfig): string {
  if (score < 1 || score > config.steps) return '';
  return config.colors[score - 1];
}

export function scoreToLabel(score: number, config: HeatmapConfig): string {
  if (score < 1 || score > config.steps) return '';
  return config.labels[score - 1];
}

export function remapScore(score: number, oldSteps: number, newSteps: number): number {
  if (oldSteps === 1) return 1;
  const normalized = (score - 1) / (oldSteps - 1);
  return Math.min(newSteps, Math.max(1, Math.round(normalized * (newSteps - 1) + 1)));
}

export function parseTags(raw: string | string[]): string[] {
  if (Array.isArray(raw)) return raw;
  if (!raw) return [];
  try {
    return JSON.parse(raw);
  } catch {
    return [];
  }
}

export function serializeTags(tags: string[]): string {
  return JSON.stringify(tags);
}

export function isValidDate(year: number, month: number, day: number): boolean {
  const d = new Date(year, month - 1, day);
  return (
    d.getFullYear() === year &&
    d.getMonth() === month - 1 &&
    d.getDate() === day
  );
}
```

- [ ] **Step 4: Run tests to verify they pass**

```bash
bun run test:run 2>&1 | tail -10
```

Expected: all tests pass, 0 failures.

- [ ] **Step 5: Commit**

```bash
git add src/lib/heatmap.ts src/lib/heatmap.test.ts
git commit -m "feat: add heatmap utility functions with tests"
```

---

## Task 6: `TagInput.svelte`

**Files:**
- Create: `src/lib/TagInput.svelte`

- [ ] **Step 1: Create the component**

```svelte
<script lang="ts">
  interface Props {
    tags: string[];
    suggestions: string[];
    onchange: (tags: string[]) => void;
  }
  let { tags, suggestions, onchange }: Props = $props();

  let inputValue = $state('');
  let showSuggestions = $state(false);

  let filteredSuggestions = $derived(
    inputValue.length > 0
      ? suggestions
          .filter(s => s.toLowerCase().includes(inputValue.toLowerCase()) && !tags.includes(s))
          .slice(0, 8)
      : []
  );

  function addTag(value: string) {
    const trimmed = value.trim().toLowerCase();
    if (trimmed && !tags.includes(trimmed)) {
      onchange([...tags, trimmed]);
    }
    inputValue = '';
    showSuggestions = false;
  }

  function removeTag(tag: string) {
    onchange(tags.filter(t => t !== tag));
  }

  function onkeydown(e: KeyboardEvent) {
    if (e.key === 'Enter' || e.key === ',') {
      e.preventDefault();
      addTag(inputValue);
    } else if (e.key === 'Backspace' && inputValue === '' && tags.length > 0) {
      removeTag(tags[tags.length - 1]);
    } else if (e.key === 'Escape') {
      showSuggestions = false;
    }
  }
</script>

<div class="tag-input-wrapper">
  <div class="tags-container">
    {#each tags as tag}
      <span class="tag">
        {tag}
        <button class="tag-remove" onclick={() => removeTag(tag)}>×</button>
      </span>
    {/each}
    <input
      type="text"
      bind:value={inputValue}
      {onkeydown}
      onfocus={() => (showSuggestions = true)}
      onblur={() => setTimeout(() => (showSuggestions = false), 150)}
      placeholder={tags.length === 0 ? 'Add symptom tags...' : ''}
      class="tag-text-input"
    />
  </div>
  {#if showSuggestions && filteredSuggestions.length > 0}
    <ul class="suggestions">
      {#each filteredSuggestions as s}
        <!-- svelte-ignore a11y_no_noninteractive_element_to_interactive_role -->
        <li role="option" aria-selected="false">
          <button onmousedown={() => addTag(s)}>{s}</button>
        </li>
      {/each}
    </ul>
  {/if}
</div>

<style>
  .tag-input-wrapper {
    position: relative;
    width: 100%;
  }
  .tags-container {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
    align-items: center;
    min-height: 36px;
    padding: 4px 8px;
    border: 1px solid var(--border, #444);
    border-radius: 6px;
    background: var(--surface, #1e1e1e);
    cursor: text;
  }
  .tag {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 2px 8px;
    border-radius: 12px;
    background: var(--accent, #3a3a5c);
    color: var(--text, #e0e0e0);
    font-size: 0.8rem;
  }
  .tag-remove {
    background: none;
    border: none;
    cursor: pointer;
    color: inherit;
    padding: 0;
    line-height: 1;
    opacity: 0.6;
  }
  .tag-remove:hover { opacity: 1; }
  .tag-text-input {
    flex: 1;
    min-width: 120px;
    background: none;
    border: none;
    outline: none;
    color: var(--text, #e0e0e0);
    font-size: 0.9rem;
  }
  .suggestions {
    position: absolute;
    top: 100%;
    left: 0;
    right: 0;
    margin: 2px 0 0;
    padding: 4px 0;
    list-style: none;
    background: var(--surface-raised, #2a2a2a);
    border: 1px solid var(--border, #444);
    border-radius: 6px;
    z-index: 100;
    max-height: 200px;
    overflow-y: auto;
  }
  .suggestions li button {
    width: 100%;
    text-align: left;
    background: none;
    border: none;
    padding: 6px 12px;
    cursor: pointer;
    color: var(--text, #e0e0e0);
    font-size: 0.9rem;
  }
  .suggestions li button:hover {
    background: var(--hover, #333);
  }
</style>
```

- [ ] **Step 2: Commit**

```bash
git add src/lib/TagInput.svelte
git commit -m "feat: add TagInput component with autocomplete"
```

---

## Task 7: `HeatmapGrid.svelte`

**Files:**
- Create: `src/lib/views/HeatmapGrid.svelte`

- [ ] **Step 1: Create the component**

```svelte
<script lang="ts">
  import type { DailyRating, HeatmapConfig } from '$lib/types';
  import { scoreToColor, isValidDate } from '$lib/heatmap';

  interface Props {
    ratings: Map<string, DailyRating>;
    config: HeatmapConfig;
    selectedDate: string | null;
    year: number;
    onselect: (date: string) => void;
  }
  let { ratings, config, selectedDate, year, onselect }: Props = $props();

  const MONTHS = ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec'];
  const DAYS = Array.from({ length: 31 }, (_, i) => i + 1);

  function dateStr(month: number, day: number): string {
    return `${year}-${String(month).padStart(2, '0')}-${String(day).padStart(2, '0')}`;
  }

  const today = new Date().toISOString().slice(0, 10);
</script>

<div class="heatmap-scroll">
  <table class="heatmap-table">
    <thead>
      <tr>
        <th class="corner"></th>
        {#each MONTHS as month}
          <th class="month-header"><span>{month}</span></th>
        {/each}
      </tr>
    </thead>
    <tbody>
      {#each DAYS as day}
        <tr>
          <td class="day-label">{day}</td>
          {#each Array.from({ length: 12 }, (_, i) => i + 1) as month}
            {@const valid = isValidDate(year, month, day)}
            {@const date = valid ? dateStr(month, day) : ''}
            {@const rating = valid ? ratings.get(date) : null}
            {@const color = rating ? scoreToColor(rating.wellness_score, config) : ''}
            {@const isToday = date === today}
            {@const isSelected = date === selectedDate}
            <td
              class="cell"
              class:invalid={!valid}
              class:today={isToday}
              class:selected={isSelected}
              class:scored={!!rating}
            >
              {#if valid}
                <button
                  style={color ? `background-color: ${color}` : ''}
                  onclick={() => onselect(date)}
                  title={date}
                ></button>
              {/if}
            </td>
          {/each}
        </tr>
      {/each}
    </tbody>
  </table>
</div>

<style>
  .heatmap-scroll {
    overflow-x: auto;
    width: 100%;
  }
  .heatmap-table {
    border-collapse: collapse;
    table-layout: fixed;
  }
  .corner {
    width: 28px;
  }
  .month-header {
    width: 36px;
    padding: 0 2px 4px;
    text-align: center;
    font-size: 0.75rem;
    color: var(--text-muted, #888);
    white-space: nowrap;
  }
  .month-header span {
    display: inline-block;
    transform: rotate(-45deg);
    transform-origin: bottom left;
    padding-bottom: 4px;
  }
  .day-label {
    padding: 1px 6px 1px 0;
    text-align: right;
    font-size: 0.75rem;
    color: var(--text-muted, #888);
    width: 28px;
  }
  .cell {
    padding: 1px;
    width: 36px;
    height: 28px;
  }
  .cell button {
    display: block;
    width: 100%;
    height: 100%;
    border: 1px solid var(--border, #333);
    border-radius: 3px;
    background: var(--cell-empty, #2a2a2a);
    cursor: pointer;
    padding: 0;
    transition: opacity 0.1s, transform 0.1s;
  }
  .cell button:hover {
    opacity: 0.85;
    transform: scale(1.1);
  }
  .cell.invalid button {
    display: none;
  }
  .cell.invalid {
    background: transparent;
  }
  .cell.today button {
    outline: 2px solid var(--accent, #6c63ff);
    outline-offset: 1px;
  }
  .cell.selected button {
    outline: 2px solid white;
    outline-offset: 1px;
  }
</style>
```

- [ ] **Step 2: Commit**

```bash
git add src/lib/views/HeatmapGrid.svelte
git commit -m "feat: add HeatmapGrid calendar display component"
```

---

## Task 8: `DayEntryPanel.svelte`

**Files:**
- Create: `src/lib/views/DayEntryPanel.svelte`

- [ ] **Step 1: Create the component**

```svelte
<script lang="ts">
  import TagInput from '$lib/TagInput.svelte';
  import { scoreToColor, scoreToLabel } from '$lib/heatmap';
  import type { DailyRating, HeatmapConfig } from '$lib/types';

  interface Props {
    date: string;
    existing: DailyRating | null;
    config: HeatmapConfig;
    allTags: string[];
    onsave: (score: number, tags: string[]) => void;
    onclose: () => void;
  }
  let { date, existing, config, allTags, onsave, onclose }: Props = $props();

  let currentTags = $state<string[]>([]);

  $effect(() => {
    currentTags = existing?.tags ?? [];
  });

  function selectScore(score: number) {
    onsave(score, currentTags);
  }

  function updateTags(newTags: string[]) {
    currentTags = newTags;
    if (existing) {
      onsave(existing.wellness_score, newTags);
    }
  }

  const steps = $derived(Array.from({ length: config.steps }, (_, i) => i + 1));

  function formatDate(d: string): string {
    const [y, m, day] = d.split('-');
    return new Date(Number(y), Number(m) - 1, Number(day)).toLocaleDateString('en-US', {
      weekday: 'short', month: 'short', day: 'numeric', year: 'numeric',
    });
  }
</script>

<div class="entry-panel">
  <div class="entry-header">
    <span class="entry-date">{formatDate(date)}</span>
    <button class="close-btn" onclick={onclose} title="Close">×</button>
  </div>
  <div class="score-row">
    {#each steps as step}
      <button
        class="score-btn"
        class:selected={existing?.wellness_score === step}
        style="background-color: {scoreToColor(step, config)}"
        onclick={() => selectScore(step)}
        title={scoreToLabel(step, config)}
      >
        {scoreToLabel(step, config)}
      </button>
    {/each}
  </div>
  <TagInput tags={currentTags} suggestions={allTags} onchange={updateTags} />
</div>

<style>
  .entry-panel {
    margin-top: 16px;
    padding: 16px;
    border: 1px solid var(--border, #444);
    border-radius: 8px;
    background: var(--surface, #1e1e1e);
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  .entry-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }
  .entry-date {
    font-size: 0.9rem;
    color: var(--text-muted, #aaa);
  }
  .close-btn {
    background: none;
    border: none;
    cursor: pointer;
    color: var(--text-muted, #888);
    font-size: 1.2rem;
    line-height: 1;
    padding: 0 4px;
  }
  .close-btn:hover { color: var(--text, #e0e0e0); }
  .score-row {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
  }
  .score-btn {
    flex: 1;
    min-width: 60px;
    padding: 8px 4px;
    border: 2px solid transparent;
    border-radius: 6px;
    cursor: pointer;
    font-size: 0.8rem;
    font-weight: 500;
    color: white;
    text-shadow: 0 1px 2px rgba(0,0,0,0.5);
    transition: transform 0.1s, border-color 0.1s;
  }
  .score-btn:hover { transform: scale(1.04); }
  .score-btn.selected {
    border-color: white;
    transform: scale(1.06);
  }
</style>
```

- [ ] **Step 2: Commit**

```bash
git add src/lib/views/DayEntryPanel.svelte
git commit -m "feat: add DayEntryPanel score selector and tag input"
```

---

## Task 9: `HeatmapConfig.svelte`

**Files:**
- Create: `src/lib/views/HeatmapConfig.svelte`

- [ ] **Step 1: Create the component**

```svelte
<script lang="ts">
  import { onMount } from 'svelte';
  import type { HeatmapConfig } from '$lib/types';
  import { getHeatmapConfig, saveHeatmapConfig } from '$lib/db';

  interface Props {
    onclose: () => void;
    onupdate: (config: HeatmapConfig) => void;
  }
  let { onclose, onupdate }: Props = $props();

  let config = $state<HeatmapConfig>({ steps: 5, labels: [], colors: [] });
  let savedSteps = $state(5);
  let error = $state('');

  onMount(async () => {
    try {
      config = await getHeatmapConfig();
      savedSteps = config.steps;
    } catch (e) {
      error = String(e);
    }
  });

  function addStep() {
    if (config.steps >= 10) return;
    config = {
      steps: config.steps + 1,
      labels: [...config.labels, 'New'],
      colors: [...config.colors, '#888888'],
    };
  }

  function removeStep() {
    if (config.steps <= 2) return;
    config = {
      steps: config.steps - 1,
      labels: config.labels.slice(0, -1),
      colors: config.colors.slice(0, -1),
    };
  }

  function updateLabel(i: number, value: string) {
    const labels = [...config.labels];
    labels[i] = value;
    config = { ...config, labels };
  }

  function updateColor(i: number, value: string) {
    const colors = [...config.colors];
    colors[i] = value;
    config = { ...config, colors };
  }

  async function handleDone() {
    error = '';
    try {
      const oldSteps = config.steps !== savedSteps ? savedSteps : null;
      await saveHeatmapConfig(config, oldSteps);
      onupdate(config);
      onclose();
    } catch (e) {
      error = String(e);
    }
  }

  const stepsChanged = $derived(config.steps !== savedSteps);
  const stepIndices = $derived(Array.from({ length: config.steps }, (_, i) => i));
</script>

<div class="config-aside">
  <div class="config-header">
    <h3>Scale Settings</h3>
    <button class="done-btn" onclick={handleDone}>Done</button>
  </div>

  {#if stepsChanged}
    <p class="remap-warning">
      ⚠ Changing the number of steps will remap all existing scores proportionally.
    </p>
  {/if}

  {#if error}
    <p class="error">{error}</p>
  {/if}

  <div class="step-count-row">
    <span>Steps</span>
    <div class="step-controls">
      <button onclick={removeStep} disabled={config.steps <= 2}>−</button>
      <span class="step-number">{config.steps}</span>
      <button onclick={addStep} disabled={config.steps >= 10}>+</button>
    </div>
  </div>

  <div class="step-list">
    {#each stepIndices as i}
      <div class="step-row">
        <input
          type="color"
          value={config.colors[i]}
          oninput={(e) => updateColor(i, (e.target as HTMLInputElement).value)}
          class="color-swatch"
        />
        <input
          type="text"
          value={config.labels[i]}
          oninput={(e) => updateLabel(i, (e.target as HTMLInputElement).value)}
          placeholder="Label"
          class="label-input"
        />
      </div>
    {/each}
  </div>
</div>

<style>
  .config-aside {
    position: fixed;
    top: 0;
    right: 0;
    height: 100%;
    width: 280px;
    background: var(--surface-raised, #222);
    border-left: 1px solid var(--border, #444);
    padding: 20px 16px;
    display: flex;
    flex-direction: column;
    gap: 16px;
    z-index: 200;
    overflow-y: auto;
  }
  .config-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }
  .config-header h3 {
    margin: 0;
    font-size: 1rem;
  }
  .done-btn {
    padding: 4px 14px;
    border-radius: 6px;
    background: var(--accent, #6c63ff);
    color: white;
    border: none;
    cursor: pointer;
    font-size: 0.85rem;
  }
  .remap-warning {
    font-size: 0.8rem;
    color: var(--warning, #f1c40f);
    background: rgba(241,196,15,0.1);
    border-radius: 6px;
    padding: 8px;
    margin: 0;
  }
  .error {
    color: var(--error, #e74c3c);
    font-size: 0.8rem;
  }
  .step-count-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    font-size: 0.9rem;
  }
  .step-controls {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .step-controls button {
    width: 28px;
    height: 28px;
    border-radius: 4px;
    border: 1px solid var(--border, #444);
    background: var(--surface, #1e1e1e);
    color: var(--text, #e0e0e0);
    cursor: pointer;
    font-size: 1rem;
  }
  .step-controls button:disabled {
    opacity: 0.3;
    cursor: default;
  }
  .step-number {
    font-weight: 600;
    min-width: 20px;
    text-align: center;
  }
  .step-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .step-row {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .color-swatch {
    width: 36px;
    height: 36px;
    padding: 2px;
    border-radius: 4px;
    border: 1px solid var(--border, #444);
    background: none;
    cursor: pointer;
  }
  .label-input {
    flex: 1;
    padding: 6px 8px;
    border-radius: 4px;
    border: 1px solid var(--border, #444);
    background: var(--surface, #1e1e1e);
    color: var(--text, #e0e0e0);
    font-size: 0.85rem;
  }
</style>
```

- [ ] **Step 2: Commit**

```bash
git add src/lib/views/HeatmapConfig.svelte
git commit -m "feat: add HeatmapConfig aside for scale customization"
```

---

## Task 10: `DailyRating.svelte`

**Files:**
- Create: `src/lib/views/DailyRating.svelte`

- [ ] **Step 1: Create the top-level view**

```svelte
<script lang="ts">
  import { onMount } from 'svelte';
  import HeatmapGrid from '$lib/views/HeatmapGrid.svelte';
  import DayEntryPanel from '$lib/views/DayEntryPanel.svelte';
  import HeatmapConfig from '$lib/views/HeatmapConfig.svelte';
  import type { DailyRating, HeatmapConfig as HeatmapConfigType } from '$lib/types';
  import { getDailyRatings, saveDailyRating, getAllTags, getHeatmapConfig } from '$lib/db';
  import { DEFAULT_CONFIG } from '$lib/heatmap';

  let year = $state(new Date().getFullYear());
  let ratings = $state<Map<string, DailyRating>>(new Map());
  let selectedDate = $state<string | null>(null);
  let config = $state<HeatmapConfigType>(DEFAULT_CONFIG);
  let allTags = $state<string[]>([]);
  let configOpen = $state(false);
  let error = $state('');

  const currentYear = new Date().getFullYear();

  async function loadYear(y: number) {
    try {
      const list = await getDailyRatings(y);
      ratings = new Map(list.map(r => [r.log_date, r]));
    } catch (e) {
      error = String(e);
    }
  }

  onMount(async () => {
    try {
      [config, allTags] = await Promise.all([getHeatmapConfig(), getAllTags()]);
    } catch (e) {
      error = String(e);
    }
  });

  $effect(() => {
    loadYear(year);
  });

  async function handleSave(score: number, tags: string[]) {
    if (!selectedDate) return;
    try {
      await saveDailyRating(selectedDate, score, tags);
      const updated: DailyRating = { log_date: selectedDate, wellness_score: score, tags };
      const next = new Map(ratings);
      next.set(selectedDate, updated);
      ratings = next;
      const newTags = tags.filter(t => !allTags.includes(t));
      if (newTags.length) allTags = [...allTags, ...newTags].sort();
    } catch (e) {
      error = String(e);
    }
  }

  function handleSelect(date: string) {
    selectedDate = selectedDate === date ? null : date;
  }

  function handleConfigUpdate(newConfig: HeatmapConfigType) {
    config = newConfig;
    loadYear(year);
  }

  const selectedRating = $derived(selectedDate ? (ratings.get(selectedDate) ?? null) : null);
</script>

<svelte:window
  onkeydown={(e) => {
    if (e.key === 'Escape') selectedDate = null;
  }}
/>

<div class="daily-rating-view">
  <div class="view-header">
    <div class="year-nav">
      <button onclick={() => { year--; selectedDate = null; }} title="Previous year">‹</button>
      <span class="year-label">{year}</span>
      <button
        onclick={() => { year++; selectedDate = null; }}
        disabled={year >= currentYear}
        title="Next year"
      >›</button>
    </div>
    <button
      class="config-toggle"
      onclick={() => (configOpen = !configOpen)}
      title="Configure scale"
      class:active={configOpen}
    >⚙</button>
  </div>

  {#if error}
    <p class="error-msg">{error}</p>
  {/if}

  <HeatmapGrid {ratings} {config} {selectedDate} {year} onselect={handleSelect} />

  {#if selectedDate}
    <DayEntryPanel
      date={selectedDate}
      existing={selectedRating}
      {config}
      {allTags}
      onsave={handleSave}
      onclose={() => (selectedDate = null)}
    />
  {/if}

  {#if configOpen}
    <HeatmapConfig onclose={() => (configOpen = false)} onupdate={handleConfigUpdate} />
  {/if}
</div>

<style>
  .daily-rating-view {
    padding: 20px;
    max-width: 960px;
    margin: 0 auto;
  }
  .view-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 16px;
  }
  .year-nav {
    display: flex;
    align-items: center;
    gap: 12px;
  }
  .year-nav button {
    background: none;
    border: 1px solid var(--border, #444);
    border-radius: 4px;
    color: var(--text, #e0e0e0);
    width: 28px;
    height: 28px;
    cursor: pointer;
    font-size: 1.1rem;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .year-nav button:disabled {
    opacity: 0.3;
    cursor: default;
  }
  .year-label {
    font-size: 1.1rem;
    font-weight: 600;
    min-width: 52px;
    text-align: center;
  }
  .config-toggle {
    background: none;
    border: 1px solid var(--border, #444);
    border-radius: 6px;
    color: var(--text-muted, #888);
    width: 32px;
    height: 32px;
    cursor: pointer;
    font-size: 1rem;
  }
  .config-toggle:hover, .config-toggle.active {
    color: var(--text, #e0e0e0);
    border-color: var(--text-muted, #888);
  }
  .error-msg {
    color: var(--error, #e74c3c);
    font-size: 0.85rem;
    margin: 0 0 12px;
  }
</style>
```

- [ ] **Step 2: Commit**

```bash
git add src/lib/views/DailyRating.svelte
git commit -m "feat: add DailyRating top-level view"
```

---

## Task 11: Navigation Update and Cleanup

**Files:**
- Modify: `src/routes/+page.svelte`
- Delete: `src/lib/views/SymptomEntry.svelte`

- [ ] **Step 1: Update imports in `+page.svelte`**

Replace:
```typescript
import SymptomEntry from '$lib/views/SymptomEntry.svelte';
```
With:
```typescript
import DailyRating from '$lib/views/DailyRating.svelte';
```

Remove:
```typescript
import SymptomEditor from '$lib/views/SymptomEditor.svelte';
```

(SymptomEditor is kept in the codebase but is no longer reachable via nav — its trigger was in SymptomEntry.)

- [ ] **Step 2: Update `ActivePanel` type**

Change:
```typescript
type ActivePanel = 'labConfig' | 'symptomConfig' | 'export' | null;
```
To:
```typescript
type ActivePanel = 'labConfig' | 'export' | null;
```

- [ ] **Step 3: Update the nav group**

Change:
```typescript
    {
      label: 'Symptoms',
      items: [
        { view: 'symptoms', label: 'Log' },
      ],
    },
```
To:
```typescript
    {
      label: 'Daily',
      items: [
        { view: 'daily-rating', label: 'Log' },
      ],
    },
```

- [ ] **Step 4: Update the view render block**

Replace:
```svelte
      {:else if currentView === 'symptoms'}
        <SymptomEntry onNavigate={navigate} openSymptomConfig={() => activePanel = 'symptomConfig'} />
```
With:
```svelte
      {:else if currentView === 'daily-rating'}
        <DailyRating />
```

- [ ] **Step 5: Remove `symptomConfig` panel blocks**

Remove both the inline and overlay `symptomConfig` blocks. They are at approximately lines 310–313 and 340–346 and look like:

```svelte
    {#if activePanel === 'symptomConfig' && configPanelInline}
      <div class="config-panel-inline" style="width: {CONFIG_PANEL_WIDTH}px">
        <SymptomEditor onClose={() => activePanel = null} />
      </div>
    {/if}
```

and:

```svelte
  {#if activePanel === 'symptomConfig' && !configPanelInline}
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="panel-overlay-backdrop" onclick={() => activePanel = null} onkeydown={() => {}}></div>
    <div class="panel-overlay" style="width: {CONFIG_PANEL_WIDTH}px">
      <SymptomEditor onClose={() => activePanel = null} />
    </div>
  {/if}
```

Remove both.

- [ ] **Step 6: Delete `SymptomEntry.svelte`**

```bash
rm src/lib/views/SymptomEntry.svelte
```

- [ ] **Step 7: Type-check**

```bash
bun run check 2>&1 | tail -20
```

Expected: 0 errors.

- [ ] **Step 8: Run tests**

```bash
bun run test:run 2>&1 | tail -10
```

Expected: all 49+ tests pass.

- [ ] **Step 9: Commit**

```bash
git add src/routes/+page.svelte
git rm src/lib/views/SymptomEntry.svelte
git commit -m "feat: replace SymptomEntry with DailyRating heatmap view"
```

---

## Verification

After all tasks:

1. `bun run tauri dev` — app should launch
2. Click "Log" in the nav → see the year heatmap
3. Click any cell → DayEntryPanel appears below with score buttons and tag input
4. Click a score button → cell updates with color immediately
5. Type a tag + Enter → tag saved, appears in autocomplete next time
6. Navigate to a previous year → old data visible
7. Click gear icon → config aside opens; change a label, click Done → label updates
8. Change step count → warning banner appears; after Done, grid re-colors
9. Escape key → deselects cell
10. `bun run check` → 0 errors
11. `bun run test:run` → all pass
