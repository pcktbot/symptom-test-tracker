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
