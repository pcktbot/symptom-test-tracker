# AI Chatbot, Artifacts & Diagnoses Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a streaming Claude AI chat side panel, artifact document management with automatic lab extraction, and a first-class Diagnosis profile system to the Symptom Test Tracker.

**Architecture:** All Claude API calls go through Rust commands using `reqwest` streaming; tokens are pushed to the frontend via Tauri `app.emit("chat-token", token)`. Three new SQLite tables (artifacts, chat_messages, diagnoses) are added inline to the existing `migrate()` function. `DiagnosisAccordion` is a self-contained Svelte component placed at the top of each major view. The chat side panel follows the same inline/overlay breakpoint pattern as the existing Glossary panel.

**Tech Stack:** Rust (reqwest 0.12 streaming, pdf-extract 0.7, uuid 1, futures-util 0.3), Tauri v2 events, Svelte 5 runes, vitest

---

## File Map

**New Rust files:**
- `src-tauri/src/commands/chat.rs` — system prompt builder, memory read/write, chat history, streaming send_chat_message
- `src-tauri/src/commands/artifacts.rs` — artifact CRUD, file save, PDF extraction, lab extraction via Claude
- `src-tauri/src/commands/diagnoses.rs` — diagnosis CRUD

**Modified Rust files:**
- `src-tauri/Cargo.toml` — add reqwest, pdf-extract, uuid, futures-util
- `src-tauri/src/db.rs` — add 3 tables + 2 settings seeds to migrate()
- `src-tauri/src/commands/mod.rs` — add pub mod for chat, artifacts, diagnoses
- `src-tauri/src/lib.rs` — register ~15 new Tauri commands

**New frontend files:**
- `src/lib/views/Chat.svelte` — streaming chat side panel
- `src/lib/views/Artifacts.svelte` — artifact list, add modal, extraction review
- `src/lib/views/Diagnoses.svelte` — profile view for managing diagnoses
- `src/lib/components/DiagnosisAccordion.svelte` — self-contained collapsible accordion

**Modified frontend files:**
- `src/lib/types.ts` — add Artifact, Diagnosis, ExtractedLabResult, ExtractionResult; update View
- `src/lib/db.ts` — add invoke wrappers for all new commands
- `src/lib/utils.ts` — add extractMemoryTag(), inferContentType()
- `src/lib/utils.test.ts` — tests for new utility functions
- `src/lib/views/Settings.svelte` — add AI section (API key + chat enabled toggle)
- `src/lib/views/Dashboard.svelte` — add DiagnosisAccordion at top
- `src/lib/views/LabResults.svelte` — add DiagnosisAccordion at top
- `src/lib/views/LabEntry.svelte` — add DiagnosisAccordion at top
- `src/lib/views/Trends.svelte` — add DiagnosisAccordion at top + onNavigate prop
- `src/lib/views/SymptomEntry.svelte` — add DiagnosisAccordion at top + onNavigate prop
- `src/routes/+page.svelte` — add Chat toggle button, Artifacts/Diagnoses nav items, new views, pass onNavigate to Trends/SymptomEntry

---

## Task 1: Cargo dependencies

**Files:**
- Modify: `src-tauri/Cargo.toml`

- [ ] **Step 1: Add new dependencies**

In `src-tauri/Cargo.toml`, add to `[dependencies]`:

```toml
reqwest = { version = "0.12", features = ["json", "stream"] }
pdf-extract = "0.7"
uuid = { version = "1", features = ["v4"] }
futures-util = "0.3"
```

- [ ] **Step 2: Verify the build compiles with new deps**

```bash
cd src-tauri && cargo check 2>&1 | tail -5
```
Expected: `Finished` with no errors (first run will download crates, takes ~60s).

- [ ] **Step 3: Commit**

```bash
git add src-tauri/Cargo.toml src-tauri/Cargo.lock
git commit -m "chore: add reqwest, pdf-extract, uuid, futures-util dependencies"
```

---

## Task 2: Database migrations

**Files:**
- Modify: `src-tauri/src/db.rs`

- [ ] **Step 1: Add three new tables and settings seeds to migrate()**

In `src-tauri/src/db.rs`, inside `migrate()`, append to the existing `conn.execute_batch(...)` string (before the closing `"`):

```rust
            CREATE TABLE IF NOT EXISTS artifacts (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL,
                source_type TEXT NOT NULL DEFAULT 'paste',
                content_type TEXT NOT NULL DEFAULT 'text',
                text_content TEXT NOT NULL DEFAULT '',
                file_path TEXT NOT NULL DEFAULT '',
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE TABLE IF NOT EXISTS chat_messages (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                role TEXT NOT NULL,
                content TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE TABLE IF NOT EXISTS diagnoses (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                short_name TEXT NOT NULL DEFAULT '',
                onset_date TEXT,
                resolution_date TEXT,
                chronic INTEGER NOT NULL DEFAULT 1,
                source TEXT NOT NULL DEFAULT '',
                details TEXT NOT NULL DEFAULT '',
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            INSERT OR IGNORE INTO settings (key, value) VALUES ('anthropic_api_key', '');
            INSERT OR IGNORE INTO settings (key, value) VALUES ('chat_enabled', 'true');
```

Also add a public helper for the artifacts directory and memory file path at the bottom of `db.rs`:

```rust
pub fn artifacts_dir() -> std::path::PathBuf {
    let base = dirs::data_local_dir().expect("Could not determine local data directory");
    let dir = base.join("symptom-test-tracker").join("artifacts");
    std::fs::create_dir_all(&dir).ok();
    dir
}

pub fn memory_path() -> std::path::PathBuf {
    let base = dirs::data_local_dir().expect("Could not determine local data directory");
    base.join("symptom-test-tracker").join("memory.md")
}
```

- [ ] **Step 2: Verify the app builds and migration runs**

```bash
cd src-tauri && cargo check
```
Expected: no errors.

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/db.rs
git commit -m "feat: add artifacts, chat_messages, diagnoses tables and memory/artifacts path helpers"
```

---

## Task 3: TypeScript types, utility functions, and tests

**Files:**
- Modify: `src/lib/types.ts`
- Modify: `src/lib/utils.ts`
- Modify: `src/lib/utils.test.ts`

- [ ] **Step 1: Write failing tests for the two new utility functions**

In `src/lib/utils.test.ts`, add after existing tests:

```typescript
import { extractMemoryTag, inferContentType } from './utils';

describe('extractMemoryTag', () => {
  it('returns null memory and original text when no tag present', () => {
    const result = extractMemoryTag('Hello world');
    expect(result.memory).toBeNull();
    expect(result.displayText).toBe('Hello world');
  });

  it('extracts memory content and strips tag from display text', () => {
    const result = extractMemoryTag('Some answer.\n<memory>User has lupus</memory>\nMore text.');
    expect(result.memory).toBe('User has lupus');
    expect(result.displayText).toBe('Some answer.\nMore text.');
  });

  it('trims whitespace from display text after stripping tag', () => {
    const result = extractMemoryTag('<memory>note</memory>');
    expect(result.displayText).toBe('');
  });
});

describe('inferContentType', () => {
  it('returns pdf for .pdf files', () => {
    expect(inferContentType('report.pdf')).toBe('pdf');
    expect(inferContentType('REPORT.PDF')).toBe('pdf');
  });

  it('returns html for .html and .htm files', () => {
    expect(inferContentType('visit.html')).toBe('html');
    expect(inferContentType('visit.htm')).toBe('html');
  });

  it('returns text for all other extensions', () => {
    expect(inferContentType('notes.txt')).toBe('text');
    expect(inferContentType('data.csv')).toBe('text');
    expect(inferContentType('noextension')).toBe('text');
  });
});
```

- [ ] **Step 2: Run tests to confirm they fail**

```bash
bun run test:run
```
Expected: FAIL — `extractMemoryTag` and `inferContentType` not found.

- [ ] **Step 3: Add the utility functions to utils.ts**

In `src/lib/utils.ts`, add at the bottom:

```typescript
export function extractMemoryTag(text: string): { memory: string | null; displayText: string } {
  const match = text.match(/<memory>([\s\S]*?)<\/memory>/);
  if (!match) return { memory: null, displayText: text };
  const memory = match[1].trim();
  const displayText = text.replace(match[0], '').replace(/\n{3,}/g, '\n\n').trim();
  return { memory, displayText };
}

export function inferContentType(filename: string): 'text' | 'html' | 'pdf' {
  const ext = filename.split('.').pop()?.toLowerCase() ?? '';
  if (ext === 'pdf') return 'pdf';
  if (ext === 'html' || ext === 'htm') return 'html';
  return 'text';
}
```

- [ ] **Step 4: Run tests to confirm they pass**

```bash
bun run test:run
```
Expected: all tests PASS.

- [ ] **Step 5: Add new TypeScript interfaces to types.ts**

In `src/lib/types.ts`, add after the existing interfaces:

```typescript
export interface Artifact {
  id: number | null;
  title: string;
  source_type: 'paste' | 'file';
  content_type: 'text' | 'html' | 'pdf';
  text_content: string;
  file_path: string;
  created_at: string;
}

export interface Diagnosis {
  id: number | null;
  name: string;
  short_name: string;
  onset_date: string | null;
  resolution_date: string | null;
  chronic: boolean;
  source: string;
  details: string;
  created_at: string;
}

export interface ExtractedLabResult {
  test_name: string;
  panel: string;
  value: number | null;
  text_value: string;
  unit: string;
  ref_range_low: number | null;
  ref_range_high: number | null;
  flag: string;
}

export interface ExtractionResult {
  date: string | null;
  results: ExtractedLabResult[];
}

export interface ChatMessage {
  role: 'user' | 'assistant';
  content: string;
  created_at: string;
}
```

Also update the `View` type union:

```typescript
export type View = 'dashboard' | 'lab-results' | 'lab-entry' | 'trends' |
  'symptoms' | 'symptom-editor' | 'lab-manage' | 'export' |
  'artifacts' | 'diagnoses' | 'welcome';
```

- [ ] **Step 6: Commit**

```bash
git add src/lib/utils.ts src/lib/utils.test.ts src/lib/types.ts
git commit -m "feat: add TypeScript types and utility functions for chat, artifacts, diagnoses"
```

---

## Task 4: Frontend DB wrapper functions

**Files:**
- Modify: `src/lib/db.ts`

- [ ] **Step 1: Add invoke wrappers for all new commands**

In `src/lib/db.ts`, add after the existing export functions:

```typescript
import type {
  // ... existing imports ...
  Artifact,
  Diagnosis,
  ExtractionResult,
  ChatMessage,
} from './types';

// Diagnoses
export async function getDiagnoses(): Promise<Diagnosis[]> {
  return invoke('get_diagnoses');
}
export async function saveDiagnosis(diagnosis: Diagnosis): Promise<number> {
  return invoke('save_diagnosis', { diagnosis });
}
export async function deleteDiagnosis(id: number): Promise<void> {
  return invoke('delete_diagnosis', { id });
}

// Artifacts
export async function getArtifacts(): Promise<Artifact[]> {
  return invoke('get_artifacts');
}
export async function getArtifact(id: number): Promise<Artifact> {
  return invoke('get_artifact', { id });
}
export async function saveArtifactPaste(title: string, content: string): Promise<number> {
  return invoke('save_artifact_paste', { title, content });
}
export async function saveArtifactFile(title: string, sourcePath: string): Promise<number> {
  return invoke('save_artifact_file', { title, sourcePath });
}
export async function deleteArtifact(id: number): Promise<void> {
  return invoke('delete_artifact', { id });
}
export async function extractLabsFromArtifact(artifactId: number): Promise<ExtractionResult> {
  return invoke('extract_labs_from_artifact', { artifactId });
}

// Chat
export async function sendChatMessage(content: string): Promise<void> {
  return invoke('send_chat_message', { content });
}
export async function getChatHistory(): Promise<ChatMessage[]> {
  return invoke('get_chat_history');
}
export async function clearChatHistory(): Promise<void> {
  return invoke('clear_chat_history');
}
export async function getMemory(): Promise<string> {
  return invoke('get_memory');
}
export async function saveMemory(content: string): Promise<void> {
  return invoke('save_memory', { content });
}
```

Note: the `import type` block at the top of db.ts needs to include the new types. Update it to add `Artifact, Diagnosis, ExtractionResult, ChatMessage` alongside existing imports.

- [ ] **Step 2: Commit**

```bash
git add src/lib/db.ts
git commit -m "feat: add DB wrapper functions for diagnoses, artifacts, and chat"
```

---

## Task 5: Diagnoses Rust commands

**Files:**
- Create: `src-tauri/src/commands/diagnoses.rs`
- Modify: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Create diagnoses.rs**

Create `src-tauri/src/commands/diagnoses.rs`:

```rust
use crate::db::Database;
use rusqlite::params;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Diagnosis {
    pub id: Option<i64>,
    pub name: String,
    pub short_name: String,
    pub onset_date: Option<String>,
    pub resolution_date: Option<String>,
    pub chronic: bool,
    pub source: String,
    pub details: String,
    pub created_at: String,
}

#[tauri::command]
pub fn get_diagnoses(db: tauri::State<'_, Database>) -> Result<Vec<Diagnosis>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare(
        "SELECT id, name, short_name, onset_date, resolution_date, chronic, source, details, created_at
         FROM diagnoses ORDER BY onset_date NULLS LAST, created_at"
    ).map_err(|e| e.to_string())?;

    let rows = stmt.query_map([], |row| {
        Ok(Diagnosis {
            id: Some(row.get(0)?),
            name: row.get(1)?,
            short_name: row.get(2)?,
            onset_date: row.get(3)?,
            resolution_date: row.get(4)?,
            chronic: row.get::<_, i64>(5)? != 0,
            source: row.get(6)?,
            details: row.get(7)?,
            created_at: row.get(8)?,
        })
    }).map_err(|e| e.to_string())?
    .collect::<Result<Vec<_>, _>>()
    .map_err(|e| e.to_string())?;

    Ok(rows)
}

#[tauri::command]
pub fn save_diagnosis(db: tauri::State<'_, Database>, diagnosis: Diagnosis) -> Result<i64, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    match diagnosis.id {
        Some(id) => {
            conn.execute(
                "UPDATE diagnoses SET name=?1, short_name=?2, onset_date=?3, resolution_date=?4,
                 chronic=?5, source=?6, details=?7 WHERE id=?8",
                params![
                    diagnosis.name, diagnosis.short_name, diagnosis.onset_date,
                    diagnosis.resolution_date, diagnosis.chronic as i64,
                    diagnosis.source, diagnosis.details, id
                ],
            ).map_err(|e| e.to_string())?;
            Ok(id)
        }
        None => {
            conn.execute(
                "INSERT INTO diagnoses (name, short_name, onset_date, resolution_date, chronic, source, details)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    diagnosis.name, diagnosis.short_name, diagnosis.onset_date,
                    diagnosis.resolution_date, diagnosis.chronic as i64,
                    diagnosis.source, diagnosis.details
                ],
            ).map_err(|e| e.to_string())?;
            Ok(conn.last_insert_rowid())
        }
    }
}

#[tauri::command]
pub fn delete_diagnosis(db: tauri::State<'_, Database>, id: i64) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM diagnoses WHERE id=?1", [id])
        .map_err(|e| e.to_string())?;
    Ok(())
}
```

- [ ] **Step 2: Register the module and commands**

In `src-tauri/src/commands/mod.rs`, add:
```rust
pub mod diagnoses;
```

In `src-tauri/src/lib.rs`, add to the `invoke_handler!` macro:
```rust
commands::diagnoses::get_diagnoses,
commands::diagnoses::save_diagnosis,
commands::diagnoses::delete_diagnosis,
```

- [ ] **Step 3: Verify build**

```bash
cd src-tauri && cargo check
```
Expected: no errors.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/commands/diagnoses.rs src-tauri/src/commands/mod.rs src-tauri/src/lib.rs
git commit -m "feat: add diagnosis CRUD Rust commands"
```

---

## Task 6: Diagnoses Svelte view

**Files:**
- Create: `src/lib/views/Diagnoses.svelte`

- [ ] **Step 1: Create Diagnoses.svelte**

Create `src/lib/views/Diagnoses.svelte`:

```svelte
<script lang="ts">
  import { getDiagnoses, saveDiagnosis, deleteDiagnosis } from '$lib/db';
  import type { Diagnosis } from '$lib/types';

  let diagnoses = $state<Diagnosis[]>([]);
  let editing = $state<Diagnosis | null>(null);
  let saving = $state(false);

  function blank(): Diagnosis {
    return {
      id: null, name: '', short_name: '', onset_date: null,
      resolution_date: null, chronic: true, source: '', details: '', created_at: ''
    };
  }

  $effect(() => {
    getDiagnoses().then(d => diagnoses = d);
  });

  function startAdd() { editing = blank(); }
  function startEdit(d: Diagnosis) { editing = { ...d }; }
  function cancelEdit() { editing = null; }

  async function handleSave() {
    if (!editing || !editing.name.trim()) return;
    saving = true;
    try {
      await saveDiagnosis(editing);
      diagnoses = await getDiagnoses();
      editing = null;
    } finally {
      saving = false;
    }
  }

  async function handleDelete(id: number) {
    await deleteDiagnosis(id);
    diagnoses = await getDiagnoses();
  }
</script>

<div class="view">
  <div class="view-header">
    <h1>Profile</h1>
    <button class="btn-primary" onclick={startAdd}>+ Add diagnosis</button>
  </div>

  {#if editing !== null}
    <div class="form-card">
      <h3>{editing.id ? 'Edit' : 'New'} diagnosis</h3>
      <div class="form-grid">
        <div class="field">
          <label>Name <span class="required">*</span></label>
          <input bind:value={editing.name} placeholder="e.g. Systemic Lupus Erythematosus" />
        </div>
        <div class="field">
          <label>Short name / abbreviation</label>
          <input bind:value={editing.short_name} placeholder="e.g. SLE" />
        </div>
        <div class="field">
          <label>Onset date</label>
          <input type="date" bind:value={editing.onset_date} />
        </div>
        <div class="field field-inline">
          <label>
            <input type="checkbox" bind:checked={editing.chronic} />
            Chronic (ongoing)
          </label>
        </div>
        {#if !editing.chronic}
          <div class="field">
            <label>Resolution date</label>
            <input type="date" bind:value={editing.resolution_date} />
          </div>
        {/if}
        <div class="field full">
          <label>Source</label>
          <input bind:value={editing.source} placeholder="e.g. Dr. Smith (Rheumatology), Idiopathic, Under investigation" />
        </div>
        <div class="field full">
          <label>Personal context</label>
          <textarea bind:value={editing.details} rows="3" placeholder="How this condition manifests for you, relevant history…"></textarea>
        </div>
      </div>
      <div class="form-actions">
        <button class="btn-ghost" onclick={cancelEdit}>Cancel</button>
        <button class="btn-primary" onclick={handleSave} disabled={saving || !editing.name.trim()}>
          {saving ? 'Saving…' : 'Save'}
        </button>
      </div>
    </div>
  {/if}

  {#if diagnoses.length === 0 && !editing}
    <p class="empty">No diagnoses added yet. Add one to give the AI assistant clinical context.</p>
  {/if}

  <div class="diagnosis-list">
    {#each diagnoses as d (d.id)}
      <div class="diagnosis-card">
        <div class="diagnosis-main">
          <div class="diagnosis-name">
            {d.name}
            {#if d.short_name}<span class="short-name">{d.short_name}</span>{/if}
            {#if d.chronic}<span class="badge badge-chronic">chronic</span>{/if}
            {#if d.source?.toLowerCase().includes('idiopathic')}<span class="badge badge-idiopathic">idiopathic</span>{/if}
          </div>
          <div class="diagnosis-meta">
            {#if d.onset_date}<span>Since {d.onset_date}</span>{/if}
            {#if d.source}<span class="source">{d.source}</span>{/if}
          </div>
          {#if d.details}<p class="details">{d.details}</p>{/if}
        </div>
        <div class="diagnosis-actions">
          <button class="btn-ghost btn-sm" onclick={() => startEdit(d)}>Edit</button>
          <button class="btn-ghost btn-sm btn-danger" onclick={() => d.id && handleDelete(d.id)}>Delete</button>
        </div>
      </div>
    {/each}
  </div>
</div>

<style>
  .view { max-width: 720px; }
  .view-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 24px; }
  .view-header h1 { margin: 0; }
  .form-card { background: var(--color-surface-raised); border: 1px solid var(--color-border); border-radius: 8px; padding: 20px; margin-bottom: 24px; }
  .form-card h3 { margin: 0 0 16px; }
  .form-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 12px; }
  .field { display: flex; flex-direction: column; gap: 4px; }
  .field.full { grid-column: 1 / -1; }
  .field.field-inline { flex-direction: row; align-items: center; }
  .field label { font-size: 12px; font-weight: 500; color: var(--color-text-muted); }
  .field input[type="text"], .field input[type="date"], .field input:not([type="checkbox"]), .field textarea {
    background: var(--color-surface); border: 1px solid var(--color-border);
    border-radius: var(--radius); padding: 6px 10px; font-size: 13px; color: var(--color-text);
  }
  .field textarea { resize: vertical; }
  .required { color: var(--color-accent); }
  .form-actions { display: flex; justify-content: flex-end; gap: 8px; margin-top: 16px; }
  .empty { color: var(--color-text-muted); font-size: 14px; }
  .diagnosis-list { display: flex; flex-direction: column; gap: 10px; }
  .diagnosis-card { background: var(--color-surface-raised); border: 1px solid var(--color-border); border-radius: 8px; padding: 14px 16px; display: flex; justify-content: space-between; align-items: flex-start; gap: 16px; }
  .diagnosis-name { font-size: 14px; font-weight: 500; display: flex; align-items: center; gap: 6px; flex-wrap: wrap; margin-bottom: 4px; }
  .short-name { color: var(--color-text-muted); font-weight: 400; }
  .badge { border-radius: 4px; padding: 1px 6px; font-size: 11px; font-weight: 500; }
  .badge-chronic { background: var(--color-accent); color: white; opacity: 0.85; }
  .badge-idiopathic { background: #cc88ff22; color: #cc88ff; }
  .diagnosis-meta { font-size: 12px; color: var(--color-text-muted); display: flex; gap: 12px; }
  .details { font-size: 12px; color: var(--color-text-muted); margin: 6px 0 0; line-height: 1.5; }
  .diagnosis-actions { display: flex; gap: 6px; flex-shrink: 0; }
  .btn-sm { padding: 3px 10px; font-size: 12px; }
  .btn-danger { color: var(--color-error, #e05555); }
</style>
```

- [ ] **Step 2: Commit**

```bash
git add src/lib/views/Diagnoses.svelte
git commit -m "feat: add Diagnoses profile view"
```

---

## Task 7: DiagnosisAccordion component

**Files:**
- Create: `src/lib/components/DiagnosisAccordion.svelte`

- [ ] **Step 1: Create the accordion component**

Create directory first: `mkdir -p src/lib/components`

Create `src/lib/components/DiagnosisAccordion.svelte`:

```svelte
<script lang="ts">
  import { getDiagnoses } from '$lib/db';
  import type { Diagnosis, View } from '$lib/types';

  let { onNavigate }: { onNavigate: (view: View) => void } = $props();

  let diagnoses = $state<Diagnosis[]>([]);
  let open = $state(false);

  $effect(() => {
    getDiagnoses().then(d => diagnoses = d);
  });
</script>

{#if diagnoses.length > 0}
  <div class="accordion">
    <button class="accordion-header" onclick={() => open = !open} aria-expanded={open}>
      <div class="accordion-left">
        <span class="accordion-label">Diagnoses</span>
        <span class="count-badge">{diagnoses.length}</span>
      </div>
      <span class="chevron" class:open>{open ? '▲' : '▼'}</span>
    </button>

    {#if open}
      <div class="accordion-body">
        {#each diagnoses as d (d.id)}
          <div class="diagnosis-row">
            <div class="diagnosis-info">
              <span class="diag-name">{d.name}</span>
              {#if d.short_name}<span class="diag-short">{d.short_name}</span>{/if}
              {#if d.chronic}<span class="badge-chronic">chronic</span>{/if}
              {#if d.source?.toLowerCase().includes('idiopathic')}<span class="badge-idiopathic">idiopathic</span>{/if}
            </div>
            {#if d.onset_date}<span class="diag-date">since {d.onset_date}</span>{/if}
          </div>
        {/each}
        <div class="accordion-footer">
          <button class="link-btn" onclick={() => onNavigate('diagnoses')}>Edit in Profile →</button>
        </div>
      </div>
    {/if}
  </div>
{/if}

<style>
  .accordion { border: 1px solid var(--color-border); border-radius: 6px; margin-bottom: 20px; overflow: hidden; }
  .accordion-header { display: flex; align-items: center; justify-content: space-between; width: 100%; padding: 8px 12px; background: var(--color-surface-raised); border: none; cursor: pointer; font-size: 13px; }
  .accordion-header:hover { background: var(--color-surface); }
  .accordion-left { display: flex; align-items: center; gap: 8px; }
  .accordion-label { font-size: 11px; font-weight: 500; text-transform: uppercase; letter-spacing: 0.05em; color: var(--color-text-muted); }
  .count-badge { background: var(--color-surface); border: 1px solid var(--color-border); border-radius: 10px; padding: 1px 7px; font-size: 10px; color: var(--color-text-muted); }
  .chevron { font-size: 9px; color: var(--color-text-muted); }
  .accordion-body { border-top: 1px solid var(--color-border); padding: 10px 12px; display: flex; flex-direction: column; gap: 6px; background: var(--color-surface); }
  .diagnosis-row { display: flex; align-items: center; justify-content: space-between; }
  .diagnosis-info { display: flex; align-items: center; gap: 6px; flex-wrap: wrap; }
  .diag-name { font-size: 13px; font-weight: 500; }
  .diag-short { font-size: 12px; color: var(--color-text-muted); }
  .badge-chronic { background: var(--color-accent); color: white; border-radius: 3px; padding: 1px 5px; font-size: 10px; opacity: 0.85; }
  .badge-idiopathic { background: #cc88ff22; color: #cc88ff; border-radius: 3px; padding: 1px 5px; font-size: 10px; }
  .diag-date { font-size: 11px; color: var(--color-text-muted); flex-shrink: 0; }
  .accordion-footer { border-top: 1px solid var(--color-border); margin-top: 4px; padding-top: 8px; }
  .link-btn { background: none; border: none; color: var(--color-accent); font-size: 12px; cursor: pointer; padding: 0; }
  .link-btn:hover { text-decoration: underline; }
</style>
```

- [ ] **Step 2: Commit**

```bash
git add src/lib/components/DiagnosisAccordion.svelte
git commit -m "feat: add DiagnosisAccordion self-contained component"
```

---

## Task 8: Wire accordion into existing views and update nav

**Files:**
- Modify: `src/lib/views/Dashboard.svelte`
- Modify: `src/lib/views/LabResults.svelte`
- Modify: `src/lib/views/LabEntry.svelte`
- Modify: `src/lib/views/Trends.svelte`
- Modify: `src/lib/views/SymptomEntry.svelte`
- Modify: `src/routes/+page.svelte`

- [ ] **Step 1: Add accordion import and usage to each view**

For each of Dashboard, LabResults, LabEntry, Trends, SymptomEntry, add to the `<script>` block:

```svelte
import DiagnosisAccordion from '$lib/components/DiagnosisAccordion.svelte';
```

Add `onNavigate` to the props destructuring in Trends.svelte and SymptomEntry.svelte (they currently receive no props):
```svelte
// Add to <script> in Trends.svelte and SymptomEntry.svelte:
import type { View } from '$lib/types';
let { onNavigate }: { onNavigate: (view: View) => void } = $props();
```

Add the component at the very top of each view's template (before the first heading or content), inside the existing wrapper div:
```svelte
<DiagnosisAccordion {onNavigate} />
```

- [ ] **Step 2: Update +page.svelte nav groups and view rendering**

In `src/routes/+page.svelte`, update `navGroups`:

```typescript
const navGroups: NavGroup[] = [
  { label: 'Overview', items: [{ view: 'dashboard', label: 'Dashboard' }] },
  {
    label: 'Labs',
    items: [
      { view: 'lab-results', label: 'Results' },
      { view: 'trends', label: 'Trends' },
      { view: 'lab-manage', label: 'Manage' },
    ],
  },
  {
    label: 'Symptoms',
    items: [
      { view: 'symptoms', label: 'Log' },
      { view: 'symptom-editor', label: 'Manage' },
    ],
  },
  {
    label: 'Data',
    items: [
      { view: 'export', label: 'Export' },
      { view: 'artifacts', label: 'Artifacts' },
    ],
  },
  { label: 'Profile', items: [{ view: 'diagnoses', label: 'Diagnoses' }] },
];
```

Add imports at top of script:
```svelte
import Diagnoses from '$lib/views/Diagnoses.svelte';
import Artifacts from '$lib/views/Artifacts.svelte';
```

Add `onNavigate` prop to Trends and SymptomEntry in the template (note: the local function is `navigate`, not `onNavigate`):
```svelte
{:else if currentView === 'trends'}
  <Trends {openGlossary} onNavigate={navigate} />
{:else if currentView === 'symptoms'}
  <SymptomEntry onNavigate={navigate} />
```

Add new view cases:
```svelte
{:else if currentView === 'artifacts'}
  <Artifacts {onNavigate} />
{:else if currentView === 'diagnoses'}
  <Diagnoses />
```

- [ ] **Step 3: Verify app compiles**

```bash
bun run check
```
Expected: no TypeScript errors.

- [ ] **Step 4: Commit**

```bash
git add src/lib/views/Dashboard.svelte src/lib/views/LabResults.svelte src/lib/views/LabEntry.svelte src/lib/views/Trends.svelte src/lib/views/SymptomEntry.svelte src/routes/+page.svelte
git commit -m "feat: add DiagnosisAccordion to views, add Artifacts/Diagnoses nav items"
```

---

## Task 9: Settings AI section

**Files:**
- Modify: `src/lib/views/Settings.svelte`

- [ ] **Step 1: Add AI section with API key field and chat toggle**

In `src/lib/views/Settings.svelte`:

Add to the `onMount` async block (alongside existing settings loads):
```typescript
const [apiKey, chatEnabled] = await Promise.all([
  getSetting('anthropic_api_key'),
  getSetting('chat_enabled'),
]);
apiKeyValue = apiKey;
chatEnabledValue = chatEnabled === 'true';
```

Add new `$state` variables alongside existing ones:
```typescript
let apiKeyValue = $state('');
let chatEnabledValue = $state(true);
let apiKeySaved = $state(false);
```

Add a new `<section>` block in the settings body, above the MCP section:

```svelte
<section class="section">
  <h3>AI Assistant</h3>
  <div class="toggle-row">
    <button
      class="toggle"
      class:on={chatEnabledValue}
      onclick={async () => { chatEnabledValue = !chatEnabledValue; await setSetting('chat_enabled', chatEnabledValue ? 'true' : 'false'); }}
      role="switch"
      aria-checked={chatEnabledValue}
      aria-label="Toggle AI chat"
      disabled={loading}
    >
      <span class="toggle-knob"></span>
    </button>
    <div class="toggle-label">
      <span class="toggle-title">Enable AI chat panel</span>
      <span class="toggle-subtitle">Show the chat assistant toggle in the toolbar.</span>
    </div>
  </div>

  <div class="api-key-row" style="margin-top: 14px;">
    <label class="field-label">Anthropic API key</label>
    <div class="api-key-input-row">
      <input
        type="password"
        bind:value={apiKeyValue}
        placeholder="sk-ant-..."
        class="api-key-input"
        autocomplete="off"
      />
      <button
        class="btn-primary btn-sm"
        onclick={async () => {
          await setSetting('anthropic_api_key', apiKeyValue);
          apiKeySaved = true;
          setTimeout(() => apiKeySaved = false, 2000);
        }}
      >
        {apiKeySaved ? 'Saved ✓' : 'Save'}
      </button>
    </div>
    <span class="toggle-subtitle">Stored locally in your app database. Never sent anywhere except api.anthropic.com.</span>
  </div>
</section>
```

Add corresponding styles (append to `<style>`):
```css
.field-label { font-size: 12px; font-weight: 500; color: var(--color-text-muted); display: block; margin-bottom: 4px; }
.api-key-row { display: flex; flex-direction: column; gap: 4px; }
.api-key-input-row { display: flex; gap: 8px; }
.api-key-input { flex: 1; background: var(--color-surface); border: 1px solid var(--color-border); border-radius: var(--radius); padding: 6px 10px; font-size: 13px; color: var(--color-text); font-family: var(--font-mono); }
```

- [ ] **Step 2: Verify no TypeScript errors**

```bash
bun run check
```

- [ ] **Step 3: Commit**

```bash
git add src/lib/views/Settings.svelte
git commit -m "feat: add AI settings section with API key and chat toggle"
```

---

## Task 10: Artifacts Rust commands

**Files:**
- Create: `src-tauri/src/commands/artifacts.rs`
- Modify: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Create artifacts.rs**

Create `src-tauri/src/commands/artifacts.rs`:

```rust
use crate::db::{Database, artifacts_dir};
use rusqlite::params;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Artifact {
    pub id: i64,
    pub title: String,
    pub source_type: String,
    pub content_type: String,
    pub text_content: String,
    pub file_path: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExtractedLabResult {
    pub test_name: String,
    pub panel: String,
    pub value: Option<f64>,
    pub text_value: String,
    pub unit: String,
    pub ref_range_low: Option<f64>,
    pub ref_range_high: Option<f64>,
    pub flag: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExtractionResult {
    pub date: Option<String>,
    pub results: Vec<ExtractedLabResult>,
}

fn row_to_artifact(row: &rusqlite::Row<'_>) -> rusqlite::Result<Artifact> {
    Ok(Artifact {
        id: row.get(0)?,
        title: row.get(1)?,
        source_type: row.get(2)?,
        content_type: row.get(3)?,
        text_content: row.get(4)?,
        file_path: row.get(5)?,
        created_at: row.get(6)?,
    })
}

#[tauri::command]
pub fn get_artifacts(db: tauri::State<'_, Database>) -> Result<Vec<Artifact>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare(
        "SELECT id, title, source_type, content_type, text_content, file_path, created_at
         FROM artifacts ORDER BY created_at DESC"
    ).map_err(|e| e.to_string())?;
    stmt.query_map([], row_to_artifact)
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_artifact(db: tauri::State<'_, Database>, id: i64) -> Result<Artifact, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    conn.query_row(
        "SELECT id, title, source_type, content_type, text_content, file_path, created_at
         FROM artifacts WHERE id=?1",
        [id],
        row_to_artifact,
    ).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_artifact_paste(
    db: tauri::State<'_, Database>,
    title: String,
    content: String,
) -> Result<i64, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO artifacts (title, source_type, content_type, text_content) VALUES (?1, 'paste', 'text', ?2)",
        params![title, content],
    ).map_err(|e| e.to_string())?;
    Ok(conn.last_insert_rowid())
}

#[tauri::command]
pub fn save_artifact_file(
    db: tauri::State<'_, Database>,
    title: String,
    source_path: String,
) -> Result<i64, String> {
    let source = std::path::Path::new(&source_path);
    let ext = source.extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    let content_type = match ext.as_str() {
        "pdf" => "pdf",
        "html" | "htm" => "html",
        _ => "text",
    };
    let original_name = source.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("file");
    let stored_name = format!("{}-{}", Uuid::new_v4(), original_name);
    let dest = artifacts_dir().join(&stored_name);

    std::fs::copy(source, &dest).map_err(|e| e.to_string())?;

    let text_content = match content_type {
        "pdf" => pdf_extract::extract_text(&dest).unwrap_or_else(|_| String::new()),
        _ => std::fs::read_to_string(&dest).unwrap_or_default(),
    };

    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO artifacts (title, source_type, content_type, text_content, file_path) VALUES (?1, 'file', ?2, ?3, ?4)",
        params![title, content_type, text_content, stored_name],
    ).map_err(|e| e.to_string())?;
    Ok(conn.last_insert_rowid())
}

#[tauri::command]
pub fn delete_artifact(db: tauri::State<'_, Database>, id: i64) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    // Delete file from disk if present
    let file_path: String = conn.query_row(
        "SELECT file_path FROM artifacts WHERE id=?1", [id],
        |r| r.get(0),
    ).unwrap_or_default();
    if !file_path.is_empty() {
        let full_path = artifacts_dir().join(&file_path);
        std::fs::remove_file(full_path).ok();
    }
    conn.execute("DELETE FROM artifacts WHERE id=?1", [id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn extract_labs_from_artifact(
    db: tauri::State<'_, Database>,
    artifact_id: i64,
) -> Result<ExtractionResult, String> {
    let (text_content, api_key) = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let text: String = conn.query_row(
            "SELECT text_content FROM artifacts WHERE id=?1", [artifact_id],
            |r| r.get(0),
        ).map_err(|e| e.to_string())?;
        let key: String = conn.query_row(
            "SELECT value FROM settings WHERE key='anthropic_api_key'", [],
            |r| r.get(0),
        ).unwrap_or_default();
        (text, key)
    };

    if api_key.is_empty() {
        return Err("Anthropic API key not configured. Go to Settings → AI.".to_string());
    }
    if text_content.is_empty() {
        return Ok(ExtractionResult { date: None, results: vec![] });
    }

    let system = "You are a medical data extraction assistant. Extract all lab test results from the document. \
        Return ONLY valid JSON with this exact structure (no markdown, no explanation): \
        {\"date\":\"YYYY-MM-DD or null\",\"results\":[{\"test_name\":\"...\",\"panel\":\"...\",\
        \"value\":1.23,\"text_value\":\"\",\"unit\":\"...\",\"ref_range_low\":0.0,\
        \"ref_range_high\":5.0,\"flag\":\"N\"}]} \
        For flag: N=normal, H=high, L=low, HH=critically high, LL=critically low. \
        Use null for value if result is text-only. Use empty string for text_value if numeric.";

    let client = reqwest::Client::new();
    let body = serde_json::json!({
        "model": "claude-sonnet-4-6",
        "max_tokens": 4096,
        "system": system,
        "messages": [{"role": "user", "content": text_content}]
    });

    let response = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", &api_key)
        .header("anthropic-version", "2023-06-01")
        .json(&body)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let resp: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;
    let text = resp["content"][0]["text"].as_str().unwrap_or("{}");
    serde_json::from_str(text).map_err(|e| format!("Failed to parse extraction result: {}", e))
}
```

Note: add `use pdf_extract;` at top — the crate name uses underscore. And `use reqwest;` is implicit from Cargo.

- [ ] **Step 2: Register module and commands**

In `src-tauri/src/commands/mod.rs`, add:
```rust
pub mod artifacts;
```

In `src-tauri/src/lib.rs`, add to `invoke_handler!`:
```rust
commands::artifacts::get_artifacts,
commands::artifacts::get_artifact,
commands::artifacts::save_artifact_paste,
commands::artifacts::save_artifact_file,
commands::artifacts::delete_artifact,
commands::artifacts::extract_labs_from_artifact,
```

- [ ] **Step 3: Build check**

```bash
cd src-tauri && cargo check
```
Expected: no errors.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/commands/artifacts.rs src-tauri/src/commands/mod.rs src-tauri/src/lib.rs
git commit -m "feat: add artifact CRUD, PDF extraction, and lab extraction Rust commands"
```

---

## Task 11: Artifacts Svelte view

**Files:**
- Create: `src/lib/views/Artifacts.svelte`

- [ ] **Step 1: Create Artifacts.svelte**

Create `src/lib/views/Artifacts.svelte`:

```svelte
<script lang="ts">
  import { open } from '@tauri-apps/plugin-dialog';
  import { getArtifacts, saveArtifactPaste, saveArtifactFile, deleteArtifact, extractLabsFromArtifact, saveLabSession } from '$lib/db';
  import DiagnosisAccordion from '$lib/components/DiagnosisAccordion.svelte';
  import type { Artifact, ExtractionResult, LabSession, LabResult, View } from '$lib/types';

  let { onNavigate }: { onNavigate: (view: View) => void } = $props();

  let artifacts = $state<Artifact[]>([]);
  let showAdd = $state(false);
  let addTab = $state<'paste' | 'file'>('paste');
  let addTitle = $state('');
  let addContent = $state('');
  let saving = $state(false);
  let extracting = $state<number | null>(null);
  let extraction = $state<ExtractionResult | null>(null);
  let extractionArtifactId = $state<number | null>(null);
  let extractionDate = $state('');
  let importing = $state(false);

  $effect(() => { getArtifacts().then(a => artifacts = a); });

  function resetAdd() {
    showAdd = false; addTitle = ''; addContent = ''; addTab = 'paste';
  }

  async function handlePickFile() {
    const path = await open({ multiple: false, filters: [{ name: 'Documents', extensions: ['pdf', 'txt', 'html', 'htm'] }] });
    if (!path || typeof path !== 'string') return;
    if (!addTitle) {
      addTitle = path.split('/').pop()?.replace(/\.[^.]+$/, '') ?? '';
    }
    saving = true;
    try {
      const id = await saveArtifactFile(addTitle || 'Untitled', path);
      artifacts = await getArtifacts();
      resetAdd();
      await startExtraction(id);
    } finally { saving = false; }
  }

  async function handleSavePaste() {
    if (!addContent.trim()) return;
    saving = true;
    try {
      const id = await saveArtifactPaste(addTitle || 'Untitled', addContent);
      artifacts = await getArtifacts();
      resetAdd();
      await startExtraction(id);
    } finally { saving = false; }
  }

  async function startExtraction(id: number) {
    extracting = id;
    extractionArtifactId = id;
    try {
      const result = await extractLabsFromArtifact(id);
      extraction = result;
      extractionDate = result.date ?? new Date().toISOString().slice(0, 10);
    } catch (e) {
      extraction = null;
    } finally { extracting = null; }
  }

  async function handleImport() {
    if (!extraction || !extractionDate) return;
    importing = true;
    try {
      const session: LabSession = { id: null, test_date: extractionDate, lab_name: '', notes: 'Imported from artifact' };
      const results = extraction.results.map(r => ({
        id: null, session_id: null,
        test_name: r.test_name, panel: r.panel,
        value: r.value, text_value: r.text_value,
        unit: r.unit, ref_range_low: r.ref_range_low,
        ref_range_high: r.ref_range_high, flag: r.flag,
      }));
      await saveLabSession(session, results as any);
      extraction = null; extractionArtifactId = null;
    } finally { importing = false; }
  }

  function dismissExtraction() { extraction = null; extractionArtifactId = null; }

  async function handleDelete(id: number) {
    await deleteArtifact(id);
    artifacts = await getArtifacts();
  }

  function typeLabel(ct: string) {
    return ct === 'pdf' ? 'PDF' : ct === 'html' ? 'HTML' : 'Text';
  }
  function typeClass(ct: string) {
    return ct === 'pdf' ? 'badge-pdf' : ct === 'html' ? 'badge-html' : 'badge-text';
  }
</script>

<div class="view">
  <DiagnosisAccordion {onNavigate} />

  <div class="view-header">
    <h1>Artifacts</h1>
    <button class="btn-primary" onclick={() => showAdd = true}>+ Add</button>
  </div>

  {#if showAdd}
    <div class="add-card">
      <div class="field" style="margin-bottom:12px">
        <label>Title</label>
        <input bind:value={addTitle} placeholder="e.g. Rheumatology Follow-up — Mar 2025" />
      </div>
      <div class="tabs">
        <button class="tab" class:active={addTab === 'paste'} onclick={() => addTab = 'paste'}>Paste text</button>
        <button class="tab" class:active={addTab === 'file'} onclick={() => addTab = 'file'}>Upload file</button>
      </div>
      {#if addTab === 'paste'}
        <textarea bind:value={addContent} rows="8" placeholder="Paste your after-visit summary, lab report, or any medical document text here…"></textarea>
        <div class="form-actions">
          <button class="btn-ghost" onclick={resetAdd}>Cancel</button>
          <button class="btn-primary" onclick={handleSavePaste} disabled={saving || !addContent.trim()}>
            {saving ? 'Saving…' : 'Save & Extract'}
          </button>
        </div>
      {:else}
        <div class="dropzone" onclick={handlePickFile} role="button" tabindex="0" onkeydown={e => e.key === 'Enter' && handlePickFile()}>
          <div class="dropzone-icon">📄</div>
          <div class="dropzone-text">Click to choose a file</div>
          <div class="dropzone-sub">PDF, plain text, or HTML</div>
        </div>
        <div class="form-actions">
          <button class="btn-ghost" onclick={resetAdd}>Cancel</button>
        </div>
      {/if}
    </div>
  {/if}

  {#if extracting !== null}
    <div class="extraction-card extracting">
      <div class="extraction-status">
        <span class="spinner"></span> Extracting lab results…
      </div>
    </div>
  {/if}

  {#if extraction !== null}
    <div class="extraction-card">
      <div class="extraction-header">
        <span>Extracted <strong>{extraction.results.length}</strong> lab result{extraction.results.length !== 1 ? 's' : ''}.</span>
        <div class="date-row">
          <label>Lab date:</label>
          <input type="date" bind:value={extractionDate} />
        </div>
      </div>
      <div class="results-table">
        {#each extraction.results.slice(0, 8) as r}
          <div class="result-row">
            <span class="result-name">{r.test_name}</span>
            <span class="result-value">{r.value ?? r.text_value} {r.unit}</span>
            <span class="result-range">{r.ref_range_low ?? ''}–{r.ref_range_high ?? ''}</span>
            <span class="flag flag-{r.flag.toLowerCase()}">{r.flag}</span>
          </div>
        {/each}
        {#if extraction.results.length > 8}
          <div class="more-results">+{extraction.results.length - 8} more</div>
        {/if}
      </div>
      <div class="form-actions">
        <button class="btn-ghost" onclick={dismissExtraction}>Dismiss</button>
        <button class="btn-primary" onclick={handleImport} disabled={importing || !extractionDate}>
          {importing ? 'Importing…' : 'Import as Lab Session →'}
        </button>
      </div>
    </div>
  {/if}

  {#if artifacts.length === 0 && !showAdd}
    <p class="empty">No artifacts yet. Add an after-visit summary or lab report to get started.</p>
  {/if}

  <div class="artifact-list">
    {#each artifacts as a (a.id)}
      <div class="artifact-row">
        <div class="artifact-info">
          <div class="artifact-title">{a.title}</div>
          <div class="artifact-meta">
            <span class="type-badge {typeClass(a.content_type)}">{typeLabel(a.content_type)}</span>
            <span class="meta-text">{a.created_at.slice(0, 10)}</span>
          </div>
        </div>
        <div class="artifact-actions">
          <button class="btn-ghost btn-sm" onclick={() => startExtraction(Number(a.id))}>Re-extract</button>
          <button class="btn-ghost btn-sm btn-danger" onclick={() => a.id && handleDelete(Number(a.id))}>Delete</button>
        </div>
      </div>
    {/each}
  </div>
</div>

<style>
  .view { max-width: 800px; }
  .view-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 20px; }
  .view-header h1 { margin: 0; }
  .add-card { background: var(--color-surface-raised); border: 1px solid var(--color-border); border-radius: 8px; padding: 16px; margin-bottom: 20px; }
  .field label { font-size: 12px; font-weight: 500; color: var(--color-text-muted); display: block; margin-bottom: 4px; }
  .field input { width: 100%; background: var(--color-surface); border: 1px solid var(--color-border); border-radius: var(--radius); padding: 6px 10px; font-size: 13px; color: var(--color-text); box-sizing: border-box; }
  .tabs { display: flex; border-bottom: 1px solid var(--color-border); margin-bottom: 12px; }
  .tab { background: none; border: none; border-bottom: 2px solid transparent; padding: 6px 14px; font-size: 13px; color: var(--color-text-muted); cursor: pointer; margin-bottom: -1px; }
  .tab.active { color: var(--color-accent); border-bottom-color: var(--color-accent); }
  textarea { width: 100%; background: var(--color-surface); border: 1px solid var(--color-border); border-radius: var(--radius); padding: 8px 10px; font-size: 13px; color: var(--color-text); resize: vertical; box-sizing: border-box; }
  .dropzone { border: 2px dashed var(--color-border); border-radius: 8px; padding: 32px; text-align: center; cursor: pointer; }
  .dropzone:hover { border-color: var(--color-accent); }
  .dropzone-icon { font-size: 28px; margin-bottom: 8px; }
  .dropzone-text { font-size: 13px; color: var(--color-text); margin-bottom: 4px; }
  .dropzone-sub { font-size: 11px; color: var(--color-text-muted); }
  .form-actions { display: flex; justify-content: flex-end; gap: 8px; margin-top: 12px; }
  .extraction-card { background: var(--color-surface-raised); border: 1px solid var(--color-border); border-radius: 8px; padding: 14px 16px; margin-bottom: 20px; }
  .extracting { display: flex; align-items: center; gap: 10px; }
  .extraction-status { display: flex; align-items: center; gap: 8px; font-size: 13px; color: var(--color-text-muted); }
  .spinner { display: inline-block; width: 14px; height: 14px; border: 2px solid var(--color-border); border-top-color: var(--color-accent); border-radius: 50%; animation: spin 0.8s linear infinite; }
  @keyframes spin { to { transform: rotate(360deg); } }
  .extraction-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 10px; font-size: 13px; }
  .date-row { display: flex; align-items: center; gap: 6px; font-size: 12px; color: var(--color-text-muted); }
  .date-row input { border: 1px solid var(--color-border); border-radius: 4px; padding: 3px 6px; font-size: 12px; background: var(--color-surface); color: var(--color-text); }
  .results-table { display: flex; flex-direction: column; gap: 3px; margin-bottom: 4px; }
  .result-row { display: grid; grid-template-columns: 1fr 100px 100px 40px; gap: 8px; font-size: 12px; padding: 4px 8px; background: var(--color-surface); border-radius: 3px; }
  .result-name { font-weight: 500; }
  .result-value, .result-range { color: var(--color-text-muted); }
  .flag { font-size: 11px; font-weight: 600; text-align: right; }
  .flag-n { color: var(--color-text-muted); }
  .flag-h, .flag-hh { color: #e05555; }
  .flag-l, .flag-ll { color: #e09055; }
  .more-results { font-size: 11px; color: var(--color-text-muted); text-align: center; padding: 4px; }
  .empty { color: var(--color-text-muted); font-size: 14px; }
  .artifact-list { display: flex; flex-direction: column; gap: 6px; }
  .artifact-row { display: flex; align-items: center; justify-content: space-between; background: var(--color-surface-raised); border: 1px solid var(--color-border); border-radius: 6px; padding: 10px 14px; }
  .artifact-title { font-size: 13px; font-weight: 500; margin-bottom: 4px; }
  .artifact-meta { display: flex; align-items: center; gap: 8px; }
  .type-badge { border-radius: 3px; padding: 1px 6px; font-size: 10px; font-weight: 500; }
  .badge-pdf { background: #1a2a3a; color: #7c9cff; }
  .badge-html { background: #2a1a3a; color: #cc88ff; }
  .badge-text { background: #1a3a2a; color: #6bff8e; }
  .meta-text { font-size: 11px; color: var(--color-text-muted); }
  .artifact-actions { display: flex; gap: 6px; }
  .btn-sm { padding: 3px 10px; font-size: 12px; }
  .btn-danger { color: var(--color-error, #e05555); }
</style>
```

- [ ] **Step 2: Verify no TypeScript errors**

```bash
bun run check
```

- [ ] **Step 3: Commit**

```bash
git add src/lib/views/Artifacts.svelte
git commit -m "feat: add Artifacts view with paste/upload, extraction review, and lab import"
```

---

## Task 12: Chat Rust utilities (system prompt, memory, history)

**Files:**
- Create: `src-tauri/src/commands/chat.rs` (partial — utilities; streaming added in Task 13)

- [ ] **Step 1: Create chat.rs with system prompt builder, memory, and history commands**

Create `src-tauri/src/commands/chat.rs`:

```rust
use crate::db::{Database, memory_path};
use rusqlite::params;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Clone)]
struct ApiMessage {
    role: String,
    content: String,
}

pub fn build_system_prompt(conn: &rusqlite::Connection) -> rusqlite::Result<String> {
    let mut prompt = String::from(
        "You are a health assistant with access to the user's personal health tracking data. \
        Answer questions accurately and cite specific data points as evidence, including \
        artifact titles and dates when referencing documents. \
        If you want to remember something for future conversations, include a <memory>...</memory> block \
        in your response containing the COMPLETE desired memory state (not just new additions — the full text). \
        The block will be stripped from the displayed response."
    );

    // Memory
    let mp = memory_path();
    if mp.exists() {
        if let Ok(mem) = std::fs::read_to_string(&mp) {
            if !mem.trim().is_empty() {
                prompt.push_str("\n\n--- MEMORY ---\n");
                prompt.push_str(mem.trim());
            }
        }
    }

    // Diagnoses
    let mut dstmt = conn.prepare(
        "SELECT name, short_name, onset_date, chronic, source, details FROM diagnoses ORDER BY onset_date NULLS LAST"
    )?;
    let diagnoses: Vec<(String, String, Option<String>, bool, String, String)> = dstmt
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get::<_, i64>(3)? != 0, r.get(4)?, r.get(5)?)))?
        .filter_map(|r| r.ok())
        .collect();
    if !diagnoses.is_empty() {
        prompt.push_str("\n\n--- DIAGNOSES ---");
        for (name, short_name, onset, chronic, source, details) in &diagnoses {
            let status = if *chronic { "chronic" } else { "resolved" };
            let onset_str = onset.as_deref().map(|d| format!(", onset {d}")).unwrap_or_default();
            prompt.push_str(&format!("\n• {name} ({short_name}) — {status}{onset_str}"));
            if !source.is_empty() { prompt.push_str(&format!("\n  Source: {source}")); }
            if !details.is_empty() { prompt.push_str(&format!("\n  Context: {details}")); }
        }
    }

    // Recent labs (last 90 days)
    let mut lstmt = conn.prepare(
        "SELECT s.id, s.test_date, s.lab_name FROM lab_sessions s
         WHERE s.test_date >= date('now', '-90 days') ORDER BY s.test_date DESC"
    )?;
    let sessions: Vec<(i64, String, String)> = lstmt
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?
        .filter_map(|r| r.ok())
        .collect();
    if !sessions.is_empty() {
        prompt.push_str("\n\n--- RECENT LABS (last 90 days) ---");
        for (sid, date, lab) in &sessions {
            let lab_str = if lab.is_empty() { String::new() } else { format!(" · {lab}") };
            prompt.push_str(&format!("\n{date}{lab_str}"));
            let mut rstmt = conn.prepare(
                "SELECT test_name, panel, value, text_value, unit, ref_range_low, ref_range_high, flag
                 FROM lab_results WHERE session_id=?1 ORDER BY panel, test_name"
            )?;
            let results: Vec<String> = rstmt
                .query_map([sid], |r| {
                    let name: String = r.get(0)?;
                    let val: Option<f64> = r.get(2)?;
                    let tv: String = r.get(3)?;
                    let unit: String = r.get(4)?;
                    let flag: String = r.get(7)?;
                    let display = if let Some(v) = val { format!("{v} {unit}") } else { tv };
                    Ok(format!("  {name}: {display} [{flag}]"))
                })?
                .filter_map(|r| r.ok())
                .collect();
            for r in results { prompt.push_str(&format!("\n{r}")); }
        }
    }

    // Current abnormal flags
    let mut astmt = conn.prepare(
        "SELECT r.test_name, r.value, r.text_value, r.unit, r.flag, s.test_date
         FROM lab_results r
         INNER JOIN (SELECT test_name, MAX(s2.test_date) as max_date
                     FROM lab_results lr JOIN lab_sessions s2 ON lr.session_id=s2.id
                     GROUP BY test_name) latest ON r.test_name=latest.test_name
         INNER JOIN lab_sessions s ON r.session_id=s.id AND s.test_date=latest.max_date
         WHERE r.flag != 'N' AND r.flag != '' ORDER BY r.test_name"
    )?;
    let abnormals: Vec<String> = astmt
        .query_map([], |r| {
            let name: String = r.get(0)?;
            let val: Option<f64> = r.get(1)?;
            let tv: String = r.get(2)?;
            let unit: String = r.get(3)?;
            let flag: String = r.get(4)?;
            let date: String = r.get(5)?;
            let display = if let Some(v) = val { format!("{v} {unit}") } else { tv };
            Ok(format!("  {name}: {display} ({flag}) as of {date}"))
        })?
        .filter_map(|r| r.ok())
        .collect();
    if !abnormals.is_empty() {
        prompt.push_str("\n\n--- CURRENT ABNORMAL FLAGS ---");
        for a in abnormals { prompt.push_str(&format!("\n{a}")); }
    }

    // Recent symptoms (last 30 days)
    let mut symptom_dates: Vec<String> = conn.prepare(
        "SELECT DISTINCT log_date FROM symptom_logs WHERE log_date >= date('now', '-30 days') ORDER BY log_date DESC"
    )?.query_map([], |r| r.get(0))?.filter_map(|r| r.ok()).collect();
    if !symptom_dates.is_empty() {
        prompt.push_str("\n\n--- RECENT SYMPTOMS (last 30 days) ---");
        for date in symptom_dates.iter().take(14) {
            let (wellness, notes): (i64, String) = conn.query_row(
                "SELECT wellness_score, notes FROM daily_summaries WHERE log_date=?1",
                [date], |r| Ok((r.get(0)?, r.get(1)?)),
            ).unwrap_or((5, String::new()));
            prompt.push_str(&format!("\n{date} (wellness {wellness}/10)"));
            let mut sstmt = conn.prepare(
                "SELECT s.name, sl.severity FROM symptom_logs sl
                 JOIN symptoms s ON sl.symptom_id=s.id
                 WHERE sl.log_date=?1 AND sl.severity > 0 ORDER BY sl.severity DESC"
            )?;
            let syms: Vec<String> = sstmt
                .query_map([date], |r| Ok(format!("  {}: {}/10", r.get::<_, String>(0)?, r.get::<_, i64>(1)?)))?
                .filter_map(|r| r.ok())
                .collect();
            for s in syms { prompt.push_str(&format!("\n{s}")); }
        }
    }

    // Artifacts
    let mut artstmt = conn.prepare(
        "SELECT title, content_type, text_content, created_at FROM artifacts ORDER BY created_at DESC"
    )?;
    let arts: Vec<(String, String, String, String)> = artstmt
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))?
        .filter_map(|r| r.ok())
        .collect();
    if !arts.is_empty() {
        prompt.push_str("\n\n--- ARTIFACTS ---");
        for (title, ct, content, date) in &arts {
            prompt.push_str(&format!("\n[{title}] ({ct}, {date})\n{content}\n"));
        }
    }

    Ok(prompt)
}

#[tauri::command]
pub fn get_chat_history(db: tauri::State<'_, Database>) -> Result<Vec<ChatMessage>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare(
        "SELECT role, content, created_at FROM chat_messages ORDER BY id ASC"
    ).map_err(|e| e.to_string())?;
    stmt.query_map([], |r| Ok(ChatMessage { role: r.get(0)?, content: r.get(1)?, created_at: r.get(2)? }))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn clear_chat_history(db: tauri::State<'_, Database>) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM chat_messages", []).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn get_memory() -> Result<String, String> {
    let path = memory_path();
    if path.exists() {
        std::fs::read_to_string(path).map_err(|e| e.to_string())
    } else {
        Ok(String::new())
    }
}

#[tauri::command]
pub fn save_memory(content: String) -> Result<(), String> {
    std::fs::write(memory_path(), content).map_err(|e| e.to_string())
}
```

- [ ] **Step 2: Register module (commands will be registered in Task 13)**

In `src-tauri/src/commands/mod.rs`, add:
```rust
pub mod chat;
```

In `src-tauri/src/lib.rs`, add to `invoke_handler!`:
```rust
commands::chat::get_chat_history,
commands::chat::clear_chat_history,
commands::chat::get_memory,
commands::chat::save_memory,
```

- [ ] **Step 3: Build check**

```bash
cd src-tauri && cargo check
```
Expected: no errors.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/commands/chat.rs src-tauri/src/commands/mod.rs src-tauri/src/lib.rs
git commit -m "feat: add chat system prompt builder, memory and history Rust commands"
```

---

## Task 13: Streaming chat command

**Files:**
- Modify: `src-tauri/src/commands/chat.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Add the streaming send_chat_message command**

Add these imports at the top of `src-tauri/src/commands/chat.rs`:

```rust
use futures_util::StreamExt;
use tauri::Manager;
```

Add the `send_chat_message` command after the existing functions:

```rust
#[tauri::command]
pub async fn send_chat_message(
    app: tauri::AppHandle,
    db: tauri::State<'_, Database>,
    content: String,
) -> Result<(), String> {
    // Save user message
    {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO chat_messages (role, content) VALUES ('user', ?1)",
            [&content],
        ).map_err(|e| e.to_string())?;
    }

    // Build system prompt and gather history
    let (system_prompt, history_messages, api_key) = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let sp = build_system_prompt(&conn).map_err(|e| e.to_string())?;
        let hist: Vec<ApiMessage> = conn.prepare(
            "SELECT role, content FROM chat_messages ORDER BY id ASC"
        ).map_err(|e| e.to_string())?
        .query_map([], |r| Ok(ApiMessage { role: r.get(0)?, content: r.get(1)? }))
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();
        let key: String = conn.query_row(
            "SELECT value FROM settings WHERE key='anthropic_api_key'", [],
            |r| r.get(0),
        ).unwrap_or_default();
        (sp, hist, key)
    };

    if api_key.is_empty() {
        return Err("Anthropic API key not configured. Go to Settings → AI to add your key.".to_string());
    }

    let body = serde_json::json!({
        "model": "claude-sonnet-4-6",
        "max_tokens": 2048,
        "stream": true,
        "system": system_prompt,
        "messages": history_messages.iter().map(|m| serde_json::json!({"role": m.role, "content": m.content})).collect::<Vec<_>>()
    });

    // Spawn streaming task
    let app_clone = app.clone();
    tokio::spawn(async move {
        let result = stream_response(app_clone.clone(), body, api_key).await;
        if let Err(e) = result {
            app_clone.emit("chat-error", e).ok();
        }
    });

    Ok(())
}

async fn stream_response(
    app: tauri::AppHandle,
    body: serde_json::Value,
    api_key: String,
) -> Result<(), String> {
    let client = reqwest::Client::new();
    let response = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", &api_key)
        .header("anthropic-version", "2023-06-01")
        .json(&body)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let mut stream = response.bytes_stream();
    let mut full_text = String::new();
    let mut line_buf = String::new();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| e.to_string())?;
        line_buf.push_str(&String::from_utf8_lossy(&chunk));

        while let Some(pos) = line_buf.find('\n') {
            let line = line_buf[..pos].trim_end_matches('\r').to_string();
            line_buf = line_buf[pos + 1..].to_string();

            if let Some(data) = line.strip_prefix("data: ") {
                if let Ok(event) = serde_json::from_str::<serde_json::Value>(data) {
                    match event["type"].as_str().unwrap_or("") {
                        "content_block_delta" => {
                            if let Some(token) = event["delta"]["text"].as_str() {
                                full_text.push_str(token);
                                app.emit("chat-token", token).ok();
                            }
                        }
                        "message_stop" => {
                            save_assistant_message(&app, &full_text);
                            app.emit("chat-done", ()).ok();
                            return Ok(());
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    // Fallback if stream ends without message_stop
    if !full_text.is_empty() {
        save_assistant_message(&app, &full_text);
    }
    app.emit("chat-done", ()).ok();
    Ok(())
}

fn save_assistant_message(app: &tauri::AppHandle, content: &str) {
    use tauri::Manager;
    // Strip <memory>...</memory> before persisting so tags don't appear in future history loads
    let clean = regex_strip_memory(content);
    let db = app.state::<Database>();
    if let Ok(conn) = db.conn.lock() {
        conn.execute(
            "INSERT INTO chat_messages (role, content) VALUES ('assistant', ?1)",
            [clean.as_str()],
        ).ok();
    }
}

fn regex_strip_memory(text: &str) -> String {
    // Simple tag strip without regex crate — find first <memory> and last </memory>
    if let (Some(start), Some(end)) = (text.find("<memory>"), text.find("</memory>")) {
        if start < end {
            let before = text[..start].trim_end();
            let after = text[end + 9..].trim_start(); // 9 = len("</memory>")
            return format!("{before} {after}").trim().to_string();
        }
    }
    text.to_string()
}
```

- [ ] **Step 2: Register send_chat_message in lib.rs**

In `src-tauri/src/lib.rs`, add to `invoke_handler!`:
```rust
commands::chat::send_chat_message,
```

- [ ] **Step 3: Build check**

```bash
cd src-tauri && cargo check
```
Expected: no errors.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/commands/chat.rs src-tauri/src/lib.rs
git commit -m "feat: add streaming send_chat_message command with Tauri event emission"
```

---

## Task 14: Chat.svelte panel

**Files:**
- Create: `src/lib/views/Chat.svelte`

- [ ] **Step 1: Create Chat.svelte**

Create `src/lib/views/Chat.svelte`:

```svelte
<script lang="ts">
  import { listen } from '@tauri-apps/api/event';
  import { getChatHistory, sendChatMessage, clearChatHistory, saveMemory } from '$lib/db';
  import { extractMemoryTag } from '$lib/utils';
  import type { ChatMessage } from '$lib/types';

  let { onClose }: { onClose: () => void } = $props();

  let messages = $state<ChatMessage[]>([]);
  let streamingContent = $state('');
  let isStreaming = $state(false);
  let inputValue = $state('');
  let error = $state('');
  let messagesEl = $state<HTMLElement | undefined>(undefined);

  $effect(() => {
    const unlisteners: (() => void)[] = [];

    async function setup() {
      const history = await getChatHistory();
      messages = history;
      scrollToBottom();

      const u1 = await listen<string>('chat-token', (event) => {
        streamingContent += event.payload;
        scrollToBottom();
      });
      const u2 = await listen<void>('chat-done', async () => {
        if (streamingContent) {
          const { memory, displayText } = extractMemoryTag(streamingContent);
          if (memory) await saveMemory(memory);
          messages = [...messages, { role: 'assistant', content: displayText, created_at: new Date().toISOString() }];
          streamingContent = '';
        }
        isStreaming = false;
        scrollToBottom();
      });
      const u3 = await listen<string>('chat-error', (event) => {
        error = event.payload;
        isStreaming = false;
        streamingContent = '';
      });

      unlisteners.push(u1, u2, u3);
    }

    setup();
    return () => unlisteners.forEach(fn => fn());
  });

  function scrollToBottom() {
    setTimeout(() => {
      if (messagesEl) messagesEl.scrollTop = messagesEl.scrollHeight;
    }, 10);
  }

  async function handleSend() {
    const text = inputValue.trim();
    if (!text || isStreaming) return;
    inputValue = '';
    error = '';
    isStreaming = true;
    messages = [...messages, { role: 'user', content: text, created_at: new Date().toISOString() }];
    scrollToBottom();
    try {
      await sendChatMessage(text);
    } catch (e: any) {
      error = e?.toString() ?? 'Failed to send message';
      isStreaming = false;
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  }

  async function handleClear() {
    await clearChatHistory();
    messages = [];
    streamingContent = '';
  }
</script>

<div class="chat-panel">
  <div class="chat-header">
    <span class="chat-title">AI Assistant</span>
    <div class="chat-header-actions">
      {#if messages.length > 0}
        <button class="icon-btn" onclick={handleClear} title="Clear history">↺</button>
      {/if}
      <button class="icon-btn" onclick={onClose} title="Close">×</button>
    </div>
  </div>

  <div class="chat-messages" bind:this={messagesEl}>
    {#if messages.length === 0 && !isStreaming}
      <div class="empty-chat">
        <div class="empty-icon">✦</div>
        <p>Ask about your labs, symptoms, or any of your health data.</p>
      </div>
    {/if}

    {#each messages as msg (msg.created_at + msg.role)}
      <div class="message" class:user={msg.role === 'user'} class:assistant={msg.role === 'assistant'}>
        {#if msg.role === 'assistant'}
          <div class="avatar"></div>
        {/if}
        <div class="bubble">{msg.content}</div>
      </div>
    {/each}

    {#if isStreaming}
      <div class="message assistant">
        <div class="avatar"></div>
        <div class="bubble streaming">
          {streamingContent}<span class="cursor"></span>
        </div>
      </div>
    {/if}

    {#if error}
      <div class="error-msg">{error}</div>
    {/if}
  </div>

  <div class="chat-input-area">
    <textarea
      bind:value={inputValue}
      onkeydown={handleKeydown}
      placeholder="Ask about your health data…"
      rows="2"
      disabled={isStreaming}
    ></textarea>
    <button class="send-btn" onclick={handleSend} disabled={isStreaming || !inputValue.trim()}>↑</button>
  </div>
</div>

<style>
  .chat-panel {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--color-surface);
    border-left: 1px solid var(--color-border);
  }

  .chat-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 10px 14px;
    border-bottom: 1px solid var(--color-border);
    flex-shrink: 0;
  }

  .chat-title {
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--color-text-muted);
  }

  .chat-header-actions { display: flex; gap: 4px; }

  .icon-btn {
    background: none;
    border: none;
    color: var(--color-text-muted);
    cursor: pointer;
    font-size: 16px;
    width: 24px;
    height: 24px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 4px;
    padding: 0;
  }
  .icon-btn:hover { background: var(--color-surface-raised); color: var(--color-text); }

  .chat-messages {
    flex: 1;
    overflow-y: auto;
    padding: 14px;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .empty-chat {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: var(--color-text-muted);
    text-align: center;
    padding: 24px;
  }
  .empty-icon { font-size: 24px; margin-bottom: 10px; }
  .empty-chat p { font-size: 13px; line-height: 1.5; margin: 0; }

  .message { display: flex; gap: 8px; align-items: flex-start; }
  .message.user { flex-direction: row-reverse; }

  .avatar {
    width: 20px;
    height: 20px;
    border-radius: 50%;
    background: linear-gradient(135deg, var(--color-accent), #cc88ff);
    flex-shrink: 0;
    margin-top: 2px;
  }

  .bubble {
    max-width: 85%;
    padding: 8px 11px;
    border-radius: 8px;
    font-size: 13px;
    line-height: 1.5;
    white-space: pre-wrap;
    word-break: break-word;
  }

  .user .bubble {
    background: var(--color-accent);
    color: white;
    border-radius: 8px 8px 2px 8px;
  }

  .assistant .bubble {
    background: var(--color-surface-raised);
    color: var(--color-text);
    border-radius: 8px 8px 8px 2px;
  }

  .cursor {
    display: inline-block;
    width: 2px;
    height: 13px;
    background: var(--color-accent);
    margin-left: 2px;
    vertical-align: middle;
    animation: blink 1s step-end infinite;
  }
  @keyframes blink { 0%, 100% { opacity: 1; } 50% { opacity: 0; } }

  .error-msg {
    background: #3a1a1a;
    color: #ff8888;
    border-radius: 6px;
    padding: 8px 12px;
    font-size: 12px;
  }

  .chat-input-area {
    display: flex;
    gap: 8px;
    align-items: flex-end;
    padding: 10px 12px;
    border-top: 1px solid var(--color-border);
    flex-shrink: 0;
  }

  .chat-input-area textarea {
    flex: 1;
    background: var(--color-surface-raised);
    border: 1px solid var(--color-border);
    border-radius: 6px;
    padding: 7px 10px;
    font-size: 13px;
    color: var(--color-text);
    resize: none;
    font-family: inherit;
    line-height: 1.4;
  }
  .chat-input-area textarea:focus { outline: none; border-color: var(--color-accent); }
  .chat-input-area textarea:disabled { opacity: 0.6; }

  .send-btn {
    width: 34px;
    height: 34px;
    border-radius: 6px;
    background: var(--color-accent);
    color: white;
    border: none;
    font-size: 16px;
    cursor: pointer;
    flex-shrink: 0;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .send-btn:disabled { opacity: 0.4; cursor: not-allowed; }
</style>
```

- [ ] **Step 2: Commit**

```bash
git add src/lib/views/Chat.svelte
git commit -m "feat: add streaming Chat side panel component"
```

---

## Task 15: Wire chat panel into +page.svelte

**Files:**
- Modify: `src/routes/+page.svelte`

- [ ] **Step 1: Add chat state and import**

In `src/routes/+page.svelte`, add to the script imports:
```svelte
import Chat from '$lib/views/Chat.svelte';
import { getSetting } from '$lib/db';
```

Add state variables:
```svelte
let chatOpen = $state(false);
let chatEnabled = $state(false);

$effect(() => {
  getSetting('chat_enabled').then(v => chatEnabled = v === 'true');
});
```

- [ ] **Step 2: Add Chat toggle button to toolbar**

In the toolbar, add after the settings gear button:

```svelte
{#if chatEnabled}
  <button
    class="chat-toggle-btn"
    class:active={chatOpen}
    onclick={() => chatOpen = !chatOpen}
    title="AI Assistant"
  >
    ✦ Chat
  </button>
{/if}
```

- [ ] **Step 3: Add Chat panel to body-area alongside existing Glossary**

Update the `body-area` div to include the Chat panel (after the existing glossary-inline block):

```svelte
{#if chatOpen}
  <div class="chat-inline" style="width: 320px">
    <Chat onClose={() => chatOpen = false} />
  </div>
{/if}
```

Also add a `chat-inline` CSS class (append to existing `<style>`):
```css
.chat-inline {
  flex-shrink: 0;
  overflow: hidden;
  height: 100%;
}

.chat-toggle-btn {
  padding: 4px 10px;
  font-size: 12px;
  border: 1px solid var(--color-border);
  border-radius: var(--radius);
  background: var(--color-surface);
  color: var(--color-text-muted);
  cursor: pointer;
  display: flex;
  align-items: center;
  gap: 4px;
  transition: background 0.15s, color 0.15s, border-color 0.15s;
}
.chat-toggle-btn:hover {
  background: var(--color-surface-raised);
  color: var(--color-text);
}
.chat-toggle-btn.active {
  background: var(--color-accent);
  color: white;
  border-color: var(--color-accent);
}
```

- [ ] **Step 4: Final type check**

```bash
bun run check
```
Expected: no TypeScript errors.

- [ ] **Step 5: Commit**

```bash
git add src/routes/+page.svelte
git commit -m "feat: wire chat panel toggle into toolbar and app layout"
```

---

## Task 16: Full integration test

- [ ] **Step 1: Run the app in dev mode**

```bash
bun run tauri dev
```
Wait for the window to open.

- [ ] **Step 2: Smoke test Diagnoses**
  - Navigate to Profile → Diagnoses
  - Add a diagnosis (e.g. "Systemic Lupus Erythematosus", short name "SLE", chronic, onset 2019-01-01)
  - Verify it saves and appears in the list
  - Navigate to Dashboard — verify the DiagnosisAccordion appears collapsed at the top
  - Expand it — verify SLE appears with chronic badge and onset date
  - Click "Edit in Profile →" — verify it navigates to Diagnoses

- [ ] **Step 3: Smoke test Settings AI section**
  - Open Settings (gear icon)
  - Verify the AI section appears with API key field and chat toggle
  - Enter your Anthropic API key and click Save
  - Verify the "Saved ✓" confirmation appears

- [ ] **Step 4: Smoke test Chat panel**
  - Verify "✦ Chat" button appears in toolbar (if chat_enabled is true)
  - Click it — verify the chat panel opens as a side panel
  - Type a message and press Enter
  - Verify the message appears in the chat
  - Verify the assistant response streams in token by token
  - Close the chat panel

- [ ] **Step 5: Smoke test Artifacts**
  - Navigate to Data → Artifacts
  - Click "+ Add", paste some text (e.g. a sample lab report with TSH: 6.2 mIU/L 0.4-4.0 H)
  - Click "Save & Extract"
  - Verify the extraction review appears with the detected results
  - Verify the detected date field is editable
  - Click "Import as Lab Session →"
  - Navigate to Labs → Results and verify the imported session appears

- [ ] **Step 6: Run tests**

```bash
bun run test:run
```
Expected: all tests pass.

- [ ] **Step 7: Final commit**

```bash
git add -A
git commit -m "feat: complete AI chatbot, artifacts, and diagnoses integration"
```
