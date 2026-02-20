use crate::db::Database;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Serialize, Deserialize)]
struct ExportLabSession {
    test_date: String,
    lab_name: String,
    notes: String,
    results: Vec<ExportLabResult>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ExportLabResult {
    test_name: String,
    panel: String,
    value: Option<f64>,
    text_value: String,
    unit: String,
    ref_range_low: Option<f64>,
    ref_range_high: Option<f64>,
    flag: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ExportSymptomDay {
    date: String,
    wellness_score: i64,
    daily_notes: String,
    symptoms: Vec<ExportSymptomEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ExportSymptomEntry {
    symptom_name: String,
    category: String,
    severity: i64,
    notes: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ExportData {
    #[serde(skip_serializing_if = "Option::is_none")]
    labs: Option<Vec<ExportLabSession>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    symptoms: Option<Vec<ExportSymptomDay>>,
}

#[tauri::command]
pub fn export_data(
    db: State<Database>,
    start_date: String,
    end_date: String,
    include_labs: bool,
    include_symptoms: bool,
    format: String,
) -> Result<String, String> {
    let conn = db.conn.lock().unwrap();

    let labs = if include_labs {
        let mut stmt = conn
            .prepare(
                "SELECT id, test_date, lab_name, notes FROM lab_sessions
                 WHERE test_date >= ?1 AND test_date <= ?2
                 ORDER BY test_date",
            )
            .map_err(|e| e.to_string())?;

        let sessions: Vec<(i64, String, String, String)> = stmt
            .query_map(params![start_date, end_date], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        let mut export_sessions = Vec::new();
        for (sid, test_date, lab_name, notes) in sessions {
            let mut rstmt = conn
                .prepare(
                    "SELECT test_name, panel, value, text_value, unit, ref_range_low, ref_range_high, flag
                     FROM lab_results WHERE session_id = ?1",
                )
                .map_err(|e| e.to_string())?;
            let results = rstmt
                .query_map(params![sid], |row| {
                    Ok(ExportLabResult {
                        test_name: row.get(0)?,
                        panel: row.get(1)?,
                        value: row.get(2)?,
                        text_value: row.get(3)?,
                        unit: row.get(4)?,
                        ref_range_low: row.get(5)?,
                        ref_range_high: row.get(6)?,
                        flag: row.get(7)?,
                    })
                })
                .map_err(|e| e.to_string())?
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| e.to_string())?;

            export_sessions.push(ExportLabSession {
                test_date,
                lab_name,
                notes,
                results,
            });
        }
        Some(export_sessions)
    } else {
        None
    };

    let symptoms = if include_symptoms {
        let mut stmt = conn
            .prepare(
                "SELECT DISTINCT sl.log_date
                 FROM symptom_logs sl
                 WHERE sl.log_date >= ?1 AND sl.log_date <= ?2
                 ORDER BY sl.log_date",
            )
            .map_err(|e| e.to_string())?;
        let dates: Vec<String> = stmt
            .query_map(params![start_date, end_date], |row| row.get(0))
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        let mut export_days = Vec::new();
        for date in dates {
            let (wellness_score, daily_notes) = conn
                .query_row(
                    "SELECT wellness_score, notes FROM daily_summaries WHERE log_date = ?1",
                    params![date],
                    |row| Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?)),
                )
                .unwrap_or((5, String::new()));

            let mut sstmt = conn
                .prepare(
                    "SELECT s.name, s.category, sl.severity, sl.notes
                     FROM symptom_logs sl
                     JOIN symptoms s ON sl.symptom_id = s.id
                     WHERE sl.log_date = ?1
                     ORDER BY s.sort_order",
                )
                .map_err(|e| e.to_string())?;
            let entries = sstmt
                .query_map(params![date], |row| {
                    Ok(ExportSymptomEntry {
                        symptom_name: row.get(0)?,
                        category: row.get(1)?,
                        severity: row.get(2)?,
                        notes: row.get(3)?,
                    })
                })
                .map_err(|e| e.to_string())?
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| e.to_string())?;

            export_days.push(ExportSymptomDay {
                date,
                wellness_score,
                daily_notes,
                symptoms: entries,
            });
        }
        Some(export_days)
    } else {
        None
    };

    let data = ExportData { labs, symptoms };

    match format.as_str() {
        "csv" => export_csv(&data),
        _ => serde_json::to_string_pretty(&data).map_err(|e| e.to_string()),
    }
}

fn export_csv(data: &ExportData) -> Result<String, String> {
    let mut csv = String::new();

    if let Some(labs) = &data.labs {
        csv.push_str("Lab Results\n");
        csv.push_str("Date,Lab,Test,Panel,Value,Text Value,Unit,Ref Low,Ref High,Flag\n");
        for session in labs {
            for r in &session.results {
                csv.push_str(&format!(
                    "{},{},{},{},{},{},{},{},{},{}\n",
                    escape_csv(&session.test_date),
                    escape_csv(&session.lab_name),
                    escape_csv(&r.test_name),
                    escape_csv(&r.panel),
                    r.value.map(|v| v.to_string()).unwrap_or_default(),
                    escape_csv(&r.text_value),
                    escape_csv(&r.unit),
                    r.ref_range_low.map(|v| v.to_string()).unwrap_or_default(),
                    r.ref_range_high.map(|v| v.to_string()).unwrap_or_default(),
                    escape_csv(&r.flag),
                ));
            }
        }
    }

    if let Some(symptoms) = &data.symptoms {
        if !csv.is_empty() {
            csv.push('\n');
        }
        csv.push_str("Symptom Logs\n");
        csv.push_str("Date,Wellness Score,Daily Notes,Symptom,Category,Severity,Notes\n");
        for day in symptoms {
            for s in &day.symptoms {
                csv.push_str(&format!(
                    "{},{},{},{},{},{},{}\n",
                    escape_csv(&day.date),
                    day.wellness_score,
                    escape_csv(&day.daily_notes),
                    escape_csv(&s.symptom_name),
                    escape_csv(&s.category),
                    s.severity,
                    escape_csv(&s.notes),
                ));
            }
        }
    }

    Ok(csv)
}

fn escape_csv(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}
