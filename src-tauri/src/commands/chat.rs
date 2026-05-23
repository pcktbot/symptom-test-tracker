use crate::db::{Database, memory_path};
use serde::{Deserialize, Serialize};
use tauri::Emitter;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Clone)]
struct ApiMessage {
    role: String,
    content: String,
}

pub fn build_system_prompt(conn: &rusqlite::Connection) -> rusqlite::Result<String> {
    let mut prompt = String::from(
        "You are a health assistant with access to the user's personal health tracking data. \
        Answer questions accurately and cite specific data points as evidence, including \
        artifact titles and dates when referencing documents. \
        If you want to remember something for future conversations, include a <memory>...</memory> block \
        in your response containing the COMPLETE desired memory state (not just new additions — the full text). \
        The block will be stripped from the displayed response."
    );

    // Memory
    let mp = memory_path();
    if mp.exists() {
        if let Ok(mem) = std::fs::read_to_string(&mp) {
            if !mem.trim().is_empty() {
                prompt.push_str("\n\n--- MEMORY ---\n");
                prompt.push_str(mem.trim());
            }
        }
    }

    // Diagnoses
    let mut dstmt = conn.prepare(
        "SELECT name, short_name, onset_date, chronic, source, details FROM diagnoses ORDER BY onset_date NULLS LAST"
    )?;
    let diagnoses: Vec<(String, String, Option<String>, bool, String, String)> = dstmt
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get::<_, i64>(3)? != 0, r.get(4)?, r.get(5)?)))?
        .filter_map(|r| r.ok())
        .collect();
    if !diagnoses.is_empty() {
        prompt.push_str("\n\n--- DIAGNOSES ---");
        for (name, short_name, onset, chronic, source, details) in &diagnoses {
            let status = if *chronic { "chronic" } else { "resolved" };
            let onset_str = onset.as_deref().map(|d| format!(", onset {d}")).unwrap_or_default();
            prompt.push_str(&format!("\n• {name} ({short_name}) — {status}{onset_str}"));
            if !source.is_empty() { prompt.push_str(&format!("\n  Source: {source}")); }
            if !details.is_empty() { prompt.push_str(&format!("\n  Context: {details}")); }
        }
    }

    // Recent labs (last 90 days)
    let mut lstmt = conn.prepare(
        "SELECT s.id, s.test_date, s.lab_name FROM lab_sessions s
         WHERE s.test_date >= date('now', '-90 days') ORDER BY s.test_date DESC"
    )?;
    let sessions: Vec<(i64, String, String)> = lstmt
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?
        .filter_map(|r| r.ok())
        .collect();
    if !sessions.is_empty() {
        prompt.push_str("\n\n--- RECENT LABS (last 90 days) ---");
        for (sid, date, lab) in &sessions {
            let lab_str = if lab.is_empty() { String::new() } else { format!(" · {lab}") };
            prompt.push_str(&format!("\n{date}{lab_str}"));
            let mut rstmt = conn.prepare(
                "SELECT test_name, panel, value, text_value, unit, ref_range_low, ref_range_high, flag
                 FROM lab_results WHERE session_id=?1 ORDER BY panel, test_name"
            )?;
            let results: Vec<String> = rstmt
                .query_map([sid], |r| {
                    let name: String = r.get(0)?;
                    let val: Option<f64> = r.get(2)?;
                    let tv: String = r.get(3)?;
                    let unit: String = r.get(4)?;
                    let flag: String = r.get(7)?;
                    let display = if let Some(v) = val { format!("{v} {unit}") } else { tv };
                    Ok(format!("  {name}: {display} [{flag}]"))
                })?
                .filter_map(|r| r.ok())
                .collect();
            for r in results { prompt.push_str(&format!("\n{r}")); }
        }
    }

    // Current abnormal flags
    let mut astmt = conn.prepare(
        "SELECT r.test_name, r.value, r.text_value, r.unit, r.flag, s.test_date
         FROM lab_results r
         INNER JOIN (SELECT test_name, MAX(s2.test_date) as max_date
                     FROM lab_results lr JOIN lab_sessions s2 ON lr.session_id=s2.id
                     GROUP BY test_name) latest ON r.test_name=latest.test_name
         INNER JOIN lab_sessions s ON r.session_id=s.id AND s.test_date=latest.max_date
         WHERE r.flag != 'N' AND r.flag != '' ORDER BY r.test_name"
    )?;
    let abnormals: Vec<String> = astmt
        .query_map([], |r| {
            let name: String = r.get(0)?;
            let val: Option<f64> = r.get(1)?;
            let tv: String = r.get(2)?;
            let unit: String = r.get(3)?;
            let flag: String = r.get(4)?;
            let date: String = r.get(5)?;
            let display = if let Some(v) = val { format!("{v} {unit}") } else { tv };
            Ok(format!("  {name}: {display} ({flag}) as of {date}"))
        })?
        .filter_map(|r| r.ok())
        .collect();
    if !abnormals.is_empty() {
        prompt.push_str("\n\n--- CURRENT ABNORMAL FLAGS ---");
        for a in abnormals { prompt.push_str(&format!("\n{a}")); }
    }

    // Recent symptoms (last 30 days)
    let symptom_dates: Vec<String> = conn.prepare(
        "SELECT DISTINCT log_date FROM symptom_logs WHERE log_date >= date('now', '-30 days') ORDER BY log_date DESC"
    )?.query_map([], |r| r.get(0))?.filter_map(|r| r.ok()).collect();
    if !symptom_dates.is_empty() {
        prompt.push_str("\n\n--- RECENT SYMPTOMS (last 30 days) ---");
        for date in symptom_dates.iter().take(14) {
            let (wellness, _notes): (i64, String) = conn.query_row(
                "SELECT wellness_score, notes FROM daily_summaries WHERE log_date=?1",
                [date], |r| Ok((r.get(0)?, r.get(1)?)),
            ).unwrap_or((5, String::new()));
            prompt.push_str(&format!("\n{date} (wellness {wellness}/10)"));
            let mut sstmt = conn.prepare(
                "SELECT s.name, sl.severity FROM symptom_logs sl
                 JOIN symptoms s ON sl.symptom_id=s.id
                 WHERE sl.log_date=?1 AND sl.severity > 0 ORDER BY sl.severity DESC"
            )?;
            let syms: Vec<String> = sstmt
                .query_map([date], |r| Ok(format!("  {}: {}/10", r.get::<_, String>(0)?, r.get::<_, i64>(1)?)))?
                .filter_map(|r| r.ok())
                .collect();
            for s in syms { prompt.push_str(&format!("\n{s}")); }
        }
    }

    // Artifacts
    let mut artstmt = conn.prepare(
        "SELECT title, content_type, text_content, created_at FROM artifacts ORDER BY created_at DESC"
    )?;
    let arts: Vec<(String, String, String, String)> = artstmt
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))?
        .filter_map(|r| r.ok())
        .collect();
    if !arts.is_empty() {
        prompt.push_str("\n\n--- ARTIFACTS ---");
        for (title, ct, content, date) in &arts {
            prompt.push_str(&format!("\n[{title}] ({ct}, {date})\n{content}\n"));
        }
    }

    Ok(prompt)
}

#[tauri::command]
pub fn get_chat_history(db: tauri::State<'_, Database>) -> Result<Vec<ChatMessage>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare(
        "SELECT role, content, created_at FROM chat_messages ORDER BY id ASC"
    ).map_err(|e| e.to_string())?;
    let rows = stmt.query_map([], |r| Ok(ChatMessage { role: r.get(0)?, content: r.get(1)?, created_at: r.get(2)? }))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(rows)
}

#[tauri::command]
pub fn clear_chat_history(db: tauri::State<'_, Database>) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM chat_messages", []).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn get_memory() -> Result<String, String> {
    let path = memory_path();
    if path.exists() {
        std::fs::read_to_string(path).map_err(|e| e.to_string())
    } else {
        Ok(String::new())
    }
}

#[tauri::command]
pub fn save_memory(content: String) -> Result<(), String> {
    std::fs::write(memory_path(), content).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn send_chat_message(
    app: tauri::AppHandle,
    db: tauri::State<'_, Database>,
    content: String,
) -> Result<(), String> {
    // Save user message
    {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO chat_messages (role, content) VALUES ('user', ?1)",
            [&content],
        ).map_err(|e| e.to_string())?;
    }

    // Build system prompt and gather history
    let (system_prompt, history_messages, api_key) = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let sp = build_system_prompt(&conn).map_err(|e| e.to_string())?;
        let hist: Vec<ApiMessage> = conn.prepare(
            "SELECT role, content FROM chat_messages ORDER BY id ASC"
        ).map_err(|e| e.to_string())?
        .query_map([], |r| Ok(ApiMessage { role: r.get(0)?, content: r.get(1)? }))
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();
        let key: String = conn.query_row(
            "SELECT value FROM settings WHERE key='anthropic_api_key'", [],
            |r| r.get(0),
        ).unwrap_or_default();
        (sp, hist, key)
    };

    if api_key.is_empty() {
        return Err("Anthropic API key not configured. Go to Settings → AI to add your key.".to_string());
    }

    let body = serde_json::json!({
        "model": "claude-sonnet-4-6",
        "max_tokens": 2048,
        "stream": true,
        "system": system_prompt,
        "messages": history_messages.iter().map(|m| serde_json::json!({"role": m.role, "content": m.content})).collect::<Vec<_>>()
    });

    // Spawn streaming task
    let app_clone = app.clone();
    tokio::spawn(async move {
        let result = stream_response(app_clone.clone(), body, api_key).await;
        if let Err(e) = result {
            app_clone.emit("chat-error", e).ok();
        }
    });

    Ok(())
}

async fn stream_response(
    app: tauri::AppHandle,
    body: serde_json::Value,
    api_key: String,
) -> Result<(), String> {
    use futures_util::StreamExt;

    let client = reqwest::Client::new();
    let response = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", &api_key)
        .header("anthropic-version", "2023-06-01")
        .json(&body)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let mut stream = response.bytes_stream();
    let mut full_text = String::new();
    let mut line_buf = String::new();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| e.to_string())?;
        line_buf.push_str(&String::from_utf8_lossy(&chunk));

        while let Some(pos) = line_buf.find('\n') {
            let line = line_buf[..pos].trim_end_matches('\r').to_string();
            line_buf = line_buf[pos + 1..].to_string();

            if let Some(data) = line.strip_prefix("data: ") {
                if let Ok(event) = serde_json::from_str::<serde_json::Value>(data) {
                    match event["type"].as_str().unwrap_or("") {
                        "content_block_delta" => {
                            if let Some(token) = event["delta"]["text"].as_str() {
                                full_text.push_str(token);
                                app.emit("chat-token", token).ok();
                            }
                        }
                        "message_stop" => {
                            save_assistant_message(&app, &full_text);
                            app.emit("chat-done", ()).ok();
                            return Ok(());
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    // Fallback if stream ends without message_stop
    if !full_text.is_empty() {
        save_assistant_message(&app, &full_text);
    }
    app.emit("chat-done", ()).ok();
    Ok(())
}

fn save_assistant_message(app: &tauri::AppHandle, content: &str) {
    use tauri::Manager;
    // Strip <memory>...</memory> before persisting so tags don't appear in future history loads
    let clean = regex_strip_memory(content);
    let db = app.state::<Database>();
    let lock = db.conn.lock();
    if let Ok(conn) = lock {
        conn.execute(
            "INSERT INTO chat_messages (role, content) VALUES ('assistant', ?1)",
            [clean.as_str()],
        ).ok();
    }
}

fn regex_strip_memory(text: &str) -> String {
    // Simple tag strip without regex crate — find first <memory> and last </memory>
    if let (Some(start), Some(end)) = (text.find("<memory>"), text.rfind("</memory>")) {
        if start < end {
            let before = text[..start].trim_end();
            let after = text[end + 9..].trim_start(); // 9 = len("</memory>")
            return format!("{before} {after}").trim().to_string();
        }
    }
    text.to_string()
}
