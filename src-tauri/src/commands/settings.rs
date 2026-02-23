use crate::db::Database;
use tauri::State;

#[tauri::command]
pub fn get_setting(key: String, db: State<'_, Database>) -> Result<String, String> {
    let conn = db.conn.lock().unwrap();
    let value: String = conn
        .query_row(
            "SELECT value FROM settings WHERE key = ?1",
            [&key],
            |row| row.get(0),
        )
        .unwrap_or_default();
    Ok(value)
}

#[tauri::command]
pub fn set_setting(key: String, value: String, db: State<'_, Database>) -> Result<(), String> {
    let conn = db.conn.lock().unwrap();
    conn.execute(
        "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
        rusqlite::params![key, value],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}
