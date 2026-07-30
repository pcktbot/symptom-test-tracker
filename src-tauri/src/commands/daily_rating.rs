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
