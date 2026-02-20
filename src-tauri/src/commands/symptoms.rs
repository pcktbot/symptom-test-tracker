use crate::db::Database;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Symptom {
    pub id: Option<i64>,
    pub name: String,
    pub category: String,
    pub description: String,
    pub active: bool,
    pub sort_order: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SymptomEntry {
    pub symptom_id: i64,
    pub severity: i64,
    pub notes: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DayLog {
    pub date: String,
    pub entries: Vec<SymptomLogEntry>,
    pub wellness_score: i64,
    pub notes: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SymptomLogEntry {
    pub symptom_id: i64,
    pub symptom_name: String,
    pub category: String,
    pub severity: i64,
    pub notes: String,
}

#[tauri::command]
pub fn get_symptoms(db: State<Database>) -> Result<Vec<Symptom>, String> {
    let conn = db.conn.lock().unwrap();
    let mut stmt = conn
        .prepare("SELECT id, name, category, description, active, sort_order FROM symptoms ORDER BY sort_order")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok(Symptom {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                category: row.get(2)?,
                description: row.get(3)?,
                active: row.get::<_, i64>(4)? != 0,
                sort_order: row.get(5)?,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_symptom(db: State<Database>, symptom: Symptom) -> Result<i64, String> {
    let conn = db.conn.lock().unwrap();
    if let Some(id) = symptom.id {
        conn.execute(
            "UPDATE symptoms SET name = ?1, category = ?2, description = ?3, active = ?4, sort_order = ?5 WHERE id = ?6",
            params![symptom.name, symptom.category, symptom.description, symptom.active as i64, symptom.sort_order, id],
        )
        .map_err(|e| e.to_string())?;
        Ok(id)
    } else {
        let max_order: i64 = conn
            .query_row("SELECT COALESCE(MAX(sort_order), -1) FROM symptoms", [], |r| r.get(0))
            .map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO symptoms (name, category, description, active, sort_order) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![symptom.name, symptom.category, symptom.description, symptom.active as i64, max_order + 1],
        )
        .map_err(|e| e.to_string())?;
        Ok(conn.last_insert_rowid())
    }
}

#[tauri::command]
pub fn delete_symptom(db: State<Database>, id: i64) -> Result<(), String> {
    let conn = db.conn.lock().unwrap();
    conn.execute("DELETE FROM symptoms WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn reorder_symptoms(db: State<Database>, ids: Vec<i64>) -> Result<(), String> {
    let conn = db.conn.lock().unwrap();
    for (i, id) in ids.iter().enumerate() {
        conn.execute(
            "UPDATE symptoms SET sort_order = ?1 WHERE id = ?2",
            params![i as i64, id],
        )
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn get_symptom_log(db: State<Database>, date: String) -> Result<DayLog, String> {
    let conn = db.conn.lock().unwrap();

    let mut stmt = conn
        .prepare(
            "SELECT sl.symptom_id, s.name, s.category, sl.severity, sl.notes
             FROM symptom_logs sl
             JOIN symptoms s ON sl.symptom_id = s.id
             WHERE sl.log_date = ?1
             ORDER BY s.sort_order",
        )
        .map_err(|e| e.to_string())?;
    let entries = stmt
        .query_map(params![date], |row| {
            Ok(SymptomLogEntry {
                symptom_id: row.get(0)?,
                symptom_name: row.get(1)?,
                category: row.get(2)?,
                severity: row.get(3)?,
                notes: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let (wellness_score, notes) = conn
        .query_row(
            "SELECT wellness_score, notes FROM daily_summaries WHERE log_date = ?1",
            params![date],
            |row| Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?)),
        )
        .unwrap_or((5, String::new()));

    Ok(DayLog {
        date,
        entries,
        wellness_score,
        notes,
    })
}

#[tauri::command]
pub fn save_symptom_log(
    db: State<Database>,
    date: String,
    entries: Vec<SymptomEntry>,
    wellness: i64,
    notes: String,
) -> Result<(), String> {
    let conn = db.conn.lock().unwrap();

    conn.execute("DELETE FROM symptom_logs WHERE log_date = ?1", params![date])
        .map_err(|e| e.to_string())?;

    for entry in &entries {
        if entry.severity > 0 || !entry.notes.is_empty() {
            conn.execute(
                "INSERT INTO symptom_logs (log_date, symptom_id, severity, notes) VALUES (?1, ?2, ?3, ?4)",
                params![date, entry.symptom_id, entry.severity, entry.notes],
            )
            .map_err(|e| e.to_string())?;
        }
    }

    conn.execute(
        "INSERT OR REPLACE INTO daily_summaries (log_date, wellness_score, notes) VALUES (?1, ?2, ?3)",
        params![date, wellness, notes],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}
