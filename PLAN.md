# Symptom Test Tracker - Full Build Plan

## Context
Greenfield Tauri v2 + Svelte 5 + Bun + Rust + SQLite desktop app for tracking lupus/APS diagnosis. Patient needs to log lab results across many panels, track daily symptoms, see abnormal values at a glance, trend results over time, export data, and expose an MCP server so an LLM can query their health data.

Repo: /Users/davidmiller/Documents/projects/symptom-test-tracker (empty git repo)

## Stack
- Frontend: Svelte 5, Bun, Vite, chart.js
- Backend: Tauri v2, Rust, rusqlite (direct, not tauri-plugin-sql)
- MCP: rmcp crate (official Rust SDK), stdio transport, separate binary
- Theme: minimal light, no background colors, all palette exposed as CSS custom properties

---

## Step 1: Scaffold

```bash
cd /Users/davidmiller/Documents/projects/symptom-test-tracker
bunx create-tauri-app . --template svelte-ts --identifier com.symptomtracker.app --manager bun --yes
bun install
```

---

## Step 2: Cargo.toml

`src-tauri/Cargo.toml` — add to [dependencies]:
```toml
rusqlite = { version = "0.31", features = ["bundled"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tauri = { version = "2", features = [] }
tauri-build = { version = "2", features = [] }
dirs = "5"

[[bin]]
name = "mcp-server"
path = "src/bin/mcp_server.rs"

[dependencies.rmcp]
version = "0.1"
features = ["server", "transport-io"]
```

---

## Step 3: Database Schema

File: `src-tauri/src/db.rs`

Tables:
- `lab_sessions` — id, test_date, lab_name, notes, created_at
- `lab_results` — id, session_id, test_name, panel, value REAL, text_value, unit, ref_range_low, ref_range_high, flag (H/L/HH/LL/N), created_at
- `symptoms` — id, name, category, description, active, sort_order
- `symptom_logs` — id, log_date, symptom_id, severity (0-10), notes, UNIQUE(log_date, symptom_id)
- `daily_summaries` — id, log_date UNIQUE, wellness_score (1-10), notes

DB path: resolved via `dirs::data_local_dir()` + `symptom-test-tracker/tracker.db`

Seed default symptoms on first run: fatigue, joint pain, joint swelling, butterfly rash, photosensitivity, hair loss, mouth sores, chest pain, shortness of breath, headache, brain fog, dry eyes, dry mouth, Raynaud's, fever, swollen lymph nodes, muscle pain, depression/anxiety

---

## Step 4: Tauri Commands

Files: `src-tauri/src/commands/labs.rs`, `symptoms.rs`, `export.rs`

### Labs
- `get_lab_sessions()` → Vec<LabSession>
- `get_lab_session(id: i64)` → LabSessionWithResults
- `save_lab_session(session: LabSession, results: Vec<LabResult>)` → i64
- `delete_lab_session(id: i64)`
- `get_latest_abnormal()` → Vec<LabResult> (latest session per test where flag != 'N')
- `get_trends(test_name: String, days: i64)` → Vec<TrendPoint>

### Symptoms
- `get_symptoms()` → Vec<Symptom>
- `save_symptom(symptom: Symptom)` → i64
- `delete_symptom(id: i64)`
- `reorder_symptoms(ids: Vec<i64>)`
- `get_symptom_log(date: String)` → DayLog
- `save_symptom_log(date: String, entries: Vec<SymptomEntry>, wellness: i64, notes: String)`

### Export
- `export_data(start_date: String, end_date: String, include_labs: bool, include_symptoms: bool, format: String)` → String (JSON or CSV)

---

## Step 5: Lab Panels & Fields

### CBC
WBC, RBC, Hemoglobin, Hematocrit, MCV, MCH, MCHC, RDW, Platelets, Neutrophils, Lymphocytes, Monocytes, Eosinophils, Basophils

### Lipids
Total Cholesterol, LDL, HDL, Triglycerides, VLDL, Non-HDL

### Inflammation
CRP, ESR, Ferritin, Fibrinogen, IL-6, Procalcitonin

### Lupus/APS Markers
ANA (titer + pattern as text), Anti-dsDNA, Anti-Smith, Anti-SSA/Ro, Anti-SSB/La, Anti-Scl-70, Anti-Jo-1, C3, C4, CH50, Anticardiolipin IgG, Anticardiolipin IgM, Beta-2 Glycoprotein I IgG, Beta-2 Glycoprotein I IgM, Lupus Anticoagulant, Direct Coombs, Urine Protein/Creatinine ratio

### Metabolic / BMP
Sodium, Potassium, Chloride, CO2, BUN, Creatinine, eGFR, Glucose, Calcium, ALT, AST, Alkaline Phosphatase, Total Bilirubin, Albumin

### Thyroid
TSH, Free T4, Free T3, TPO Antibodies

### Coagulation (critical for APS)
PT, INR, aPTT

---

## Step 6: Svelte Views

### App.svelte
- Sidebar nav (Dashboard, Lab Results, Trends, Symptoms, Export)
- Route state via simple string store (no router needed)

### Dashboard.svelte
- Calls `get_latest_abnormal()`
- Groups by panel
- Flag badges: H (amber), L (blue), HH (red), LL (red)
- Shows value, unit, reference range

### LabEntry.svelte / LabEdit.svelte
- Date picker + lab name + notes at top
- Tabs or accordion per panel
- Each field: label, value input, unit (pre-filled), ref range low/high, flag selector
- text_value field for ANA titer/pattern and Lupus Anticoagulant (qualitative results)
- Save triggers `save_lab_session`

### Trends.svelte
- Multi-select for test names (searchable)
- Date range picker
- Line chart via chart.js — one dataset per selected test
- Normalized toggle (show % of ref range midpoint) for overlaying disparate units

### SymptomEntry.svelte
- Date picker (default today)
- List of active symptoms from `get_symptoms()`
- Each: name, severity slider 0–10, optional notes
- Overall wellness score 1–10
- Daily notes textarea

### SymptomEditor.svelte
- Table of all symptoms
- Add/edit inline: name, category, description, active toggle
- Drag to reorder (or up/down arrows)

### Export.svelte
- Date range pickers
- Checkboxes: include labs, include symptoms
- Format: JSON / CSV
- Export button triggers `export_data` → opens save dialog via Tauri dialog plugin

---

## Step 7: Theme (CSS Variables)

`src/app.css`:
```css
:root {
  --color-text: #1a1a1a;
  --color-text-muted: #6b7280;
  --color-border: #e5e7eb;
  --color-border-strong: #d1d5db;
  --color-surface: #ffffff;
  --color-surface-raised: #f9fafb;
  --color-accent: #2563eb;
  --color-accent-hover: #1d4ed8;
  --color-flag-high: #d97706;
  --color-flag-low: #2563eb;
  --color-flag-critical: #dc2626;
  --color-flag-normal: #16a34a;
  --color-success: #16a34a;
  --color-warning: #d97706;
  --color-danger: #dc2626;
  --font-sans: system-ui, -apple-system, sans-serif;
  --font-mono: ui-monospace, monospace;
  --radius: 6px;
  --shadow-sm: 0 1px 2px rgba(0,0,0,0.05);
  --shadow: 0 1px 4px rgba(0,0,0,0.08);
}
```

No background colors on body. Borders for separation. Clean typography. Compact density appropriate for data-heavy medical views.

---

## Step 8: MCP Server Binary

File: `src-tauri/src/bin/mcp_server.rs`

Opens DB read-only at same path as main app.
Uses `rmcp` with stdio transport.

### Tools exposed:
- `get_recent_labs` — params: `{ days: number }` — returns lab results from last N days grouped by session
- `get_abnormal_labs` — no params — returns all currently flagged abnormal values (latest per test)
- `get_symptom_history` — params: `{ days: number }` — returns symptom logs with severity
- `get_trends` — params: `{ test_name: string, days: number }` — returns time series for one test
- `get_daily_summaries` — params: `{ days: number }` — returns wellness scores and daily notes

Claude Desktop config users would add:
```json
{
  "mcpServers": {
    "symptom-tracker": {
      "command": "/Applications/symptom-test-tracker.app/Contents/MacOS/mcp-server"
    }
  }
}
```

---

## File Structure

```
symptom-test-tracker/
├── package.json
├── bun.lockb
├── vite.config.ts
├── svelte.config.js
├── index.html
├── src/
│   ├── app.css
│   ├── App.svelte
│   ├── main.ts
│   ├── lib/
│   │   ├── db.ts           # invoke() wrappers
│   │   ├── types.ts        # TS interfaces
│   │   └── utils.ts        # flag formatting, date helpers
│   └── views/
│       ├── Dashboard.svelte
│       ├── LabResults.svelte
│       ├── LabEntry.svelte
│       ├── Trends.svelte
│       ├── SymptomEntry.svelte
│       ├── SymptomEditor.svelte
│       └── Export.svelte
└── src-tauri/
    ├── Cargo.toml
    ├── build.rs
    ├── tauri.conf.json
    └── src/
        ├── main.rs
        ├── lib.rs
        ├── db.rs
        ├── commands/
        │   ├── mod.rs
        │   ├── labs.rs
        │   ├── symptoms.rs
        │   └── export.rs
        └── bin/
            └── mcp_server.rs
```

---

## Verification
1. `bun run tauri dev` — app opens, can navigate all views
2. Enter a lab session with abnormal values → Dashboard shows them flagged
3. Enter 3+ sessions for one test → Trends view shows line chart
4. Log symptoms for several days → SymptomEntry shows history
5. Export → produces valid JSON/CSV with correct date range
6. MCP: `echo '{"jsonrpc":"2.0","method":"tools/list","id":1}' | ./mcp-server` returns tool list
