use crate::db::Database;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LabSession {
    pub id: Option<i64>,
    pub test_date: String,
    pub lab_name: String,
    pub notes: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LabResult {
    pub id: Option<i64>,
    pub session_id: Option<i64>,
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
pub struct LabSessionWithResults {
    pub session: LabSession,
    pub results: Vec<LabResult>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TrendPoint {
    pub test_date: String,
    pub value: Option<f64>,
    pub text_value: String,
    pub flag: String,
    pub ref_range_low: Option<f64>,
    pub ref_range_high: Option<f64>,
}

#[tauri::command]
pub fn get_lab_sessions(db: State<Database>) -> Result<Vec<LabSession>, String> {
    let conn = db.conn.lock().unwrap();
    let mut stmt = conn
        .prepare("SELECT id, test_date, lab_name, notes FROM lab_sessions ORDER BY test_date DESC")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok(LabSession {
                id: Some(row.get(0)?),
                test_date: row.get(1)?,
                lab_name: row.get(2)?,
                notes: row.get(3)?,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_lab_session(db: State<Database>, id: i64) -> Result<LabSessionWithResults, String> {
    let conn = db.conn.lock().unwrap();
    let session = conn
        .query_row(
            "SELECT id, test_date, lab_name, notes FROM lab_sessions WHERE id = ?1",
            params![id],
            |row| {
                Ok(LabSession {
                    id: Some(row.get(0)?),
                    test_date: row.get(1)?,
                    lab_name: row.get(2)?,
                    notes: row.get(3)?,
                })
            },
        )
        .map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT id, session_id, test_name, panel, value, text_value, unit, ref_range_low, ref_range_high, flag
             FROM lab_results WHERE session_id = ?1 ORDER BY panel, test_name",
        )
        .map_err(|e| e.to_string())?;
    let results = stmt
        .query_map(params![id], |row| {
            Ok(LabResult {
                id: Some(row.get(0)?),
                session_id: Some(row.get(1)?),
                test_name: row.get(2)?,
                panel: row.get(3)?,
                value: row.get(4)?,
                text_value: row.get(5)?,
                unit: row.get(6)?,
                ref_range_low: row.get(7)?,
                ref_range_high: row.get(8)?,
                flag: row.get(9)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(LabSessionWithResults { session, results })
}

#[tauri::command]
pub fn save_lab_session(
    db: State<Database>,
    session: LabSession,
    results: Vec<LabResult>,
) -> Result<i64, String> {
    let conn = db.conn.lock().unwrap();

    let session_id = if let Some(id) = session.id {
        conn.execute(
            "UPDATE lab_sessions SET test_date = ?1, lab_name = ?2, notes = ?3 WHERE id = ?4",
            params![session.test_date, session.lab_name, session.notes, id],
        )
        .map_err(|e| e.to_string())?;
        conn.execute("DELETE FROM lab_results WHERE session_id = ?1", params![id])
            .map_err(|e| e.to_string())?;
        id
    } else {
        conn.execute(
            "INSERT INTO lab_sessions (test_date, lab_name, notes) VALUES (?1, ?2, ?3)",
            params![session.test_date, session.lab_name, session.notes],
        )
        .map_err(|e| e.to_string())?;
        conn.last_insert_rowid()
    };

    for r in &results {
        if r.value.is_none() && r.text_value.is_empty() {
            continue;
        }
        conn.execute(
            "INSERT INTO lab_results (session_id, test_name, panel, value, text_value, unit, ref_range_low, ref_range_high, flag)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                session_id,
                r.test_name,
                r.panel,
                r.value,
                r.text_value,
                r.unit,
                r.ref_range_low,
                r.ref_range_high,
                r.flag,
            ],
        )
        .map_err(|e| e.to_string())?;
    }

    Ok(session_id)
}

#[tauri::command]
pub fn delete_lab_session(db: State<Database>, id: i64) -> Result<(), String> {
    let conn = db.conn.lock().unwrap();
    conn.execute("DELETE FROM lab_sessions WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn get_latest_abnormal(db: State<Database>) -> Result<Vec<LabResult>, String> {
    let conn = db.conn.lock().unwrap();
    let mut stmt = conn
        .prepare(
            "SELECT r.id, r.session_id, r.test_name, r.panel, r.value, r.text_value, r.unit,
                    r.ref_range_low, r.ref_range_high, r.flag
             FROM lab_results r
             INNER JOIN (
                 SELECT test_name, MAX(s.test_date) as max_date
                 FROM lab_results lr
                 JOIN lab_sessions s ON lr.session_id = s.id
                 GROUP BY test_name
             ) latest ON r.test_name = latest.test_name
             INNER JOIN lab_sessions s ON r.session_id = s.id AND s.test_date = latest.max_date
             WHERE r.flag != 'N' AND r.flag != ''
             ORDER BY r.panel, r.test_name",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok(LabResult {
                id: Some(row.get(0)?),
                session_id: Some(row.get(1)?),
                test_name: row.get(2)?,
                panel: row.get(3)?,
                value: row.get(4)?,
                text_value: row.get(5)?,
                unit: row.get(6)?,
                ref_range_low: row.get(7)?,
                ref_range_high: row.get(8)?,
                flag: row.get(9)?,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_trends(
    db: State<Database>,
    test_name: String,
    days: i64,
) -> Result<Vec<TrendPoint>, String> {
    let conn = db.conn.lock().unwrap();
    let mut stmt = conn
        .prepare(
            "SELECT s.test_date, r.value, r.text_value, r.flag, r.ref_range_low, r.ref_range_high
             FROM lab_results r
             JOIN lab_sessions s ON r.session_id = s.id
             WHERE r.test_name = ?1
               AND s.test_date >= date('now', '-' || ?2 || ' days')
             ORDER BY s.test_date ASC",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(params![test_name, days], |row| {
            Ok(TrendPoint {
                test_date: row.get(0)?,
                value: row.get(1)?,
                text_value: row.get(2)?,
                flag: row.get(3)?,
                ref_range_low: row.get(4)?,
                ref_range_high: row.get(5)?,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_all_test_names(db: State<Database>) -> Result<Vec<String>, String> {
    let conn = db.conn.lock().unwrap();
    let mut stmt = conn
        .prepare("SELECT DISTINCT test_name FROM lab_results ORDER BY test_name")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| row.get(0))
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}
