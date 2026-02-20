use rusqlite::{Connection, Result, params};
use std::path::PathBuf;
use std::sync::Mutex;

pub struct Database {
    pub conn: Mutex<Connection>,
}

impl Database {
    pub fn new() -> Result<Self> {
        let path = db_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("Failed to create data directory");
        }
        let conn = Connection::open(&path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
        let db = Database { conn: Mutex::new(conn) };
        db.migrate()?;
        Ok(db)
    }

    fn migrate(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS lab_sessions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                test_date TEXT NOT NULL,
                lab_name TEXT NOT NULL DEFAULT '',
                notes TEXT NOT NULL DEFAULT '',
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE TABLE IF NOT EXISTS lab_results (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id INTEGER NOT NULL REFERENCES lab_sessions(id) ON DELETE CASCADE,
                test_name TEXT NOT NULL,
                panel TEXT NOT NULL DEFAULT '',
                value REAL,
                text_value TEXT NOT NULL DEFAULT '',
                unit TEXT NOT NULL DEFAULT '',
                ref_range_low REAL,
                ref_range_high REAL,
                flag TEXT NOT NULL DEFAULT 'N',
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE TABLE IF NOT EXISTS symptoms (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                category TEXT NOT NULL DEFAULT '',
                description TEXT NOT NULL DEFAULT '',
                active INTEGER NOT NULL DEFAULT 1,
                sort_order INTEGER NOT NULL DEFAULT 0
            );

            CREATE TABLE IF NOT EXISTS symptom_logs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                log_date TEXT NOT NULL,
                symptom_id INTEGER NOT NULL REFERENCES symptoms(id) ON DELETE CASCADE,
                severity INTEGER NOT NULL DEFAULT 0,
                notes TEXT NOT NULL DEFAULT '',
                UNIQUE(log_date, symptom_id)
            );

            CREATE TABLE IF NOT EXISTS daily_summaries (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                log_date TEXT NOT NULL UNIQUE,
                wellness_score INTEGER NOT NULL DEFAULT 5,
                notes TEXT NOT NULL DEFAULT ''
            );

            CREATE INDEX IF NOT EXISTS idx_lab_results_session ON lab_results(session_id);
            CREATE INDEX IF NOT EXISTS idx_lab_results_test ON lab_results(test_name);
            CREATE INDEX IF NOT EXISTS idx_symptom_logs_date ON symptom_logs(log_date);
            "
        )?;

        // Seed default symptoms if table is empty
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM symptoms", [], |r| r.get(0))?;
        if count == 0 {
            let defaults = vec![
                ("Fatigue", "General"),
                ("Joint Pain", "Musculoskeletal"),
                ("Joint Swelling", "Musculoskeletal"),
                ("Butterfly Rash", "Skin"),
                ("Photosensitivity", "Skin"),
                ("Hair Loss", "Skin"),
                ("Mouth Sores", "Mucosal"),
                ("Chest Pain", "Cardiovascular"),
                ("Shortness of Breath", "Cardiovascular"),
                ("Headache", "Neurological"),
                ("Brain Fog", "Neurological"),
                ("Dry Eyes", "Mucosal"),
                ("Dry Mouth", "Mucosal"),
                ("Raynaud's", "Vascular"),
                ("Fever", "General"),
                ("Swollen Lymph Nodes", "General"),
                ("Muscle Pain", "Musculoskeletal"),
                ("Depression/Anxiety", "Neurological"),
            ];
            for (i, (name, category)) in defaults.iter().enumerate() {
                conn.execute(
                    "INSERT INTO symptoms (name, category, sort_order) VALUES (?1, ?2, ?3)",
                    params![name, category, i as i64],
                )?;
            }
        }

        Ok(())
    }
}

pub fn db_path() -> PathBuf {
    let base = dirs::data_local_dir().expect("Could not determine local data directory");
    base.join("symptom-test-tracker").join("tracker.db")
}
