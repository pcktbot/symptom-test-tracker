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
    let rows = stmt.query_map([], row_to_artifact)
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string());
    rows
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
