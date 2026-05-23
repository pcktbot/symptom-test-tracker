# Design: AI Chatbot, Artifacts, and Diagnoses

**Date:** 2026-05-18  
**Status:** Approved

---

## Overview

Three new systems added to the Symptom Test Tracker desktop app:

1. **AI Chat panel** — streaming Claude assistant that knows the user's full health context, opens as a side panel alongside any view
2. **Artifacts** — manage medical documents (after-visit summaries, lab reports) with automatic lab extraction
3. **Diagnoses (Profile)** — first-class diagnosis objects that anchor clinical context for the AI and appear as a collapsible accordion in every view

---

## Architecture

### New Rust command modules

- `src-tauri/src/commands/chat.rs` — API key management, streaming chat, system prompt construction, memory read/write
- `src-tauri/src/commands/artifacts.rs` — artifact CRUD, PDF text extraction, lab extraction via Claude
- `src-tauri/src/commands/diagnoses.rs` — diagnosis CRUD

All registered in `lib.rs` via `invoke_handler!`, same as existing commands.

### New frontend views

- `src/lib/views/Chat.svelte` — streaming chat side panel
- `src/lib/views/Artifacts.svelte` — artifact list, add modal, extraction review
- `src/lib/views/Diagnoses.svelte` — profile view for managing diagnoses
- `src/lib/components/DiagnosisAccordion.svelte` — shared collapsible accordion used in other views

### Nav changes

**Toolbar:** Add a "✦ Chat" toggle button (right side, near settings gear). Toggles the chat side panel open/closed from any view.

**Nav groups:**
- Data group: add **Artifacts** alongside Export
- New **Profile** group: single **Diagnoses** item

---

## Data Layer

### New SQLite tables (added to `db.rs` `migrate()`)

```sql
CREATE TABLE IF NOT EXISTS artifacts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    source_type TEXT NOT NULL DEFAULT 'paste',   -- 'paste' | 'file'
    content_type TEXT NOT NULL DEFAULT 'text',   -- 'text' | 'html' | 'pdf'
    text_content TEXT NOT NULL DEFAULT '',        -- extracted/pasted text
    file_path TEXT NOT NULL DEFAULT '',           -- filename only; full path resolved via dirs::data_local_dir()
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS chat_messages (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    role TEXT NOT NULL,    -- 'user' | 'assistant'
    content TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS diagnoses (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,                   -- full pathology name
    short_name TEXT NOT NULL DEFAULT '',  -- abbreviation / common name
    onset_date TEXT,                      -- YYYY-MM-DD, NULL if unknown
    resolution_date TEXT,                 -- NULL if ongoing
    chronic INTEGER NOT NULL DEFAULT 1,
    source TEXT NOT NULL DEFAULT '',      -- e.g. "Dr. Smith (Rheumatology)", "Idiopathic", "Under investigation"
    details TEXT NOT NULL DEFAULT '',     -- personal context / how it manifests for this user
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

INSERT OR IGNORE INTO settings (key, value) VALUES ('anthropic_api_key', '');
INSERT OR IGNORE INTO settings (key, value) VALUES ('chat_enabled', 'true');
```

### File storage

Uploaded artifact files are saved to:
```
~/Library/Application Support/symptom-test-tracker/artifacts/<uuid>-<original-filename>
```
A UUID is generated before the DB insert so the filename is known before the row is created. The `file_path` column stores only `<uuid>-<original-filename>`; Rust resolves the full path using `dirs::data_local_dir()` — same pattern as `db_path()`. Original files are always preserved so a PDF viewer can be added later. For paste artifacts, `file_path` is stored as an empty string.

### Memory file

A single markdown file at:
```
~/Library/Application Support/symptom-test-tracker/memory.md
```
Read by Rust at chat session start and injected into the system prompt. Updated by Claude during conversation (see Memory Update below).

---

## System Prompt Construction

Built fresh at the start of each conversation. Assembled in Rust from live DB queries:

```
You are a health assistant with access to [user]'s personal health tracking data.
Answer questions accurately and cite specific data points as evidence, including
artifact titles and dates when drawing on document content.

--- MEMORY ---
[contents of memory.md, if non-empty]

--- DIAGNOSES ---
• [name] ([short_name]) — [chronic|resolved], onset [date]
  Source: [source]
  Context: [details]
...

--- RECENT LABS (last 90 days) ---
[lab sessions with all results, flags, reference ranges]

--- CURRENT ABNORMAL FLAGS ---
[latest flagged result per test, with previous value for comparison]

--- RECENT SYMPTOMS (last 30 days) ---
[daily logs with severity per symptom and wellness scores]

--- ARTIFACTS ---
[for each artifact: title, date, content_type, full text_content]
```

Chat history from `chat_messages` is passed as the `messages` array. The system prompt is not repeated per message.

---

## Chat Panel

### UI

- Toggled by a "✦ Chat" button in the toolbar (right side, near the settings gear)
- Opens as a side panel — same inline/overlay breakpoint logic as the Glossary (`bodyWidth >= CONTENT_MAX + CHAT_WIDTH + 48`)
- Panel width: ~300px
- Layout: header ("AI Assistant" label + close button), scrollable message list, input bar
- User messages: right-aligned, dark blue background
- Assistant messages: left-aligned, dark surface background, small gradient avatar dot
- Streaming response shows a blinking cursor while tokens are arriving

### Streaming token flow

1. User submits message → `invoke('send_chat_message', { content })` (fire-and-forget, no awaited response)
2. Rust saves user message to `chat_messages`, then opens a streaming `reqwest` POST to `api.anthropic.com/v1/messages` with `"stream": true`
3. Rust reads SSE response line by line; for each `content_block_delta` event: `app_handle.emit("chat-token", token_text)`
4. On `message_stop`: Rust assembles full response, saves to `chat_messages`, emits `chat-done`
5. Frontend: `listen("chat-token", ...)` appends each chunk to a `$state` string for the current message; `listen("chat-done", ...)` finalizes it and clears the streaming state

### Memory updates

Claude may include a `<memory>...</memory>` block anywhere in its response. The system prompt instructs Claude that the block must contain the **complete desired memory state** (not a delta) — Rust does a full overwrite of `memory.md` on each update. The frontend:
- Strips the block from the displayed message text before rendering
- Calls `invoke('save_memory', { content })` with the extracted content
- Rust overwrites `memory.md` entirely with the new content

---

## Artifacts

### Views / states

**List view** (`Artifacts.svelte`):
- Sorted by `created_at` descending
- Each row: title, date, type badge (PDF / Text / HTML), extracted lab count ("14 labs extracted" or "no labs extracted")
- "+ Add" button opens the add modal

**Add modal** — two tabs:
- **Paste text**: textarea for plain text or HTML content; always stored with `content_type = 'text'` regardless of whether the pasted content contains HTML markup
- **Upload file**: drag-and-drop zone accepting PDF, plain text, HTML; Tauri `plugin-dialog` for the file picker; `content_type` is inferred from the file extension (`.pdf` → `'pdf'`, `.html`/`.htm` → `'html'`, everything else → `'text'`)
- Title field (required)
- Save button labeled "Save & Extract"

**Extraction review** — appears immediately after saving:
- Claude-branded header: "Extracted N lab results from [title]"
- Detected date shown (extracted by Claude or defaulting to artifact save date); user can edit before import
- Results table: test name, value+unit, reference range, flag
- "Import as Lab Session →" creates a `lab_sessions` row with the confirmed date and inserts all results; "Dismiss" skips import
- No foreign key between artifact and lab session — they are associated by date

### PDF text extraction

Uses the `pdf-extract` crate in Rust. Extracted text stored in `text_content`. Original file always retained on disk.

### Lab extraction command

`extract_labs_from_artifact(artifact_id)` — Rust command that:
1. Fetches `text_content` from SQLite
2. Calls Claude with a structured extraction prompt requesting JSON: `{ date, results: [{ test_name, panel, value, unit, ref_range_low, ref_range_high, flag, text_value }] }`
3. Returns the parsed JSON to the frontend for review (not yet saved to DB at this point)

---

## Diagnoses (Profile)

### Profile view (`Diagnoses.svelte`)

Full-width view under the Profile nav group. Displays a list of diagnoses; "+ Add" opens an inline form with fields:
- Name (required)
- Short name
- Onset date (date picker, optional)
- Resolution date (optional; hidden if chronic is checked)
- Chronic toggle
- Source
- Details (textarea)

### DiagnosisAccordion component

`src/lib/components/DiagnosisAccordion.svelte` — included at the top of:
Dashboard, LabResults, LabEntry, Trends, SymptomEntry, Artifacts

The component is self-contained: it fetches its own diagnoses via `invoke('get_diagnoses')` on mount. Parent views pass only an `onNavigate` callback for the "Edit in Profile →" link.

**Collapsed state (default):** Single row — label "Diagnoses" + count badge + chevron  
**Expanded state:** One row per diagnosis showing:
- Full name, short name (muted), `chronic` badge, `idiopathic` badge (if source contains "idiopathic")
- Onset date right-aligned
- "Edit in Profile →" link at the bottom that navigates to the Diagnoses view

---

## Settings additions

The existing Settings modal gains an **AI** section:
- Anthropic API key field (password input, saved to `settings` table)
- Chat enabled toggle (maps to `chat_enabled` setting)

---

## New Cargo dependencies

```toml
reqwest = { version = "0.12", features = ["json", "stream"] }
pdf-extract = "0.7"
futures-util = "0.3"
```

(`tokio` and `serde`/`serde_json` already present)

---

## New TypeScript types

```ts
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
  date: string;
  results: ExtractedLabResult[];
}

export type View = 'dashboard' | 'lab-results' | 'lab-entry' | 'trends' |
  'symptoms' | 'symptom-editor' | 'lab-manage' | 'export' |
  'artifacts' | 'diagnoses' | 'welcome';
```
