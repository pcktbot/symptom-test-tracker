use std::path::PathBuf;
use rmcp::{
    ErrorData as McpError, ServerHandler, ServiceExt,
    handler::server::{tool::ToolRouter, wrapper::Parameters},
    model::*,
    schemars, tool, tool_handler, tool_router,
    transport::stdio,
};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

fn db_path() -> PathBuf {
    let base = dirs::data_local_dir().expect("Could not determine local data directory");
    base.join("symptom-test-tracker").join("tracker.db")
}

fn open_db() -> Result<Connection, McpError> {
    let path = db_path();
    Connection::open_with_flags(&path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|e| McpError::internal_error(format!("Failed to open database: {}", e), None))
}

fn open_db_rw() -> Result<Connection, McpError> {
    let path = db_path();
    Connection::open_with_flags(&path, rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE)
        .map_err(|e| McpError::internal_error(format!("Failed to open database for writing: {}", e), None))
}

fn check_enabled(conn: &Connection) -> Result<(), McpError> {
    let enabled: String = conn
        .query_row("SELECT value FROM settings WHERE key = 'mcp_enabled'", [], |r| r.get(0))
        .unwrap_or_default();
    if enabled != "true" {
        return Err(McpError::internal_error(
            "MCP access is currently disabled in the Symptom Tracker app. Open the app → Settings to re-enable.".to_string(),
            None,
        ));
    }
    Ok(())
}

fn check_write_enabled(conn: &Connection) -> Result<(), McpError> {
    let enabled: String = conn
        .query_row("SELECT value FROM settings WHERE key = 'mcp_write_enabled'", [], |r| r.get(0))
        .unwrap_or_default();
    if enabled != "true" {
        return Err(McpError::internal_error(
            "MCP write access is currently disabled in the Symptom Tracker app. Open the app → Settings to enable write access.".to_string(),
            None,
        ));
    }
    Ok(())
}

// Parameter structs
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct DaysParam {
    #[schemars(description = "Number of days to look back")]
    pub days: i64,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct TrendsParam {
    #[schemars(description = "Name of the test to get trends for")]
    pub test_name: String,
    #[schemars(description = "Number of days to look back")]
    pub days: i64,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct InsertLabSessionParam {
    #[schemars(description = "Date of the lab test in YYYY-MM-DD format")]
    pub test_date: String,
    #[schemars(description = "Name of the lab/facility (optional)")]
    pub lab_name: Option<String>,
    #[schemars(description = "Free-text notes about this lab session (optional)")]
    pub notes: Option<String>,
    #[schemars(description = "Array of individual lab result entries")]
    pub results: Vec<LabResultInput>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct LabResultInput {
    #[schemars(description = "Name of the lab test (e.g. 'TSH', 'WBC', 'Hemoglobin')")]
    pub test_name: String,
    #[schemars(description = "Panel/category the test belongs to (e.g. 'Thyroid', 'CBC')")]
    pub panel: Option<String>,
    #[schemars(description = "Numeric value of the result, if applicable")]
    pub value: Option<f64>,
    #[schemars(description = "Text value for non-numeric results (e.g. 'Positive', 'Reactive')")]
    pub text_value: Option<String>,
    #[schemars(description = "Unit of measurement (e.g. 'mIU/L', 'mg/dL')")]
    pub unit: Option<String>,
    #[schemars(description = "Lower bound of the reference range")]
    pub ref_range_low: Option<f64>,
    #[schemars(description = "Upper bound of the reference range")]
    pub ref_range_high: Option<f64>,
    #[schemars(description = "Flag: 'N' for normal, 'H' for high, 'L' for low, 'A' for abnormal")]
    pub flag: Option<String>,
}

// Response structs
#[derive(Debug, Serialize)]
struct LabSessionResult {
    test_date: String,
    lab_name: String,
    notes: String,
    results: Vec<LabResultRow>,
}

#[derive(Debug, Serialize)]
struct LabResultRow {
    test_name: String,
    panel: String,
    value: Option<f64>,
    text_value: String,
    unit: String,
    ref_range_low: Option<f64>,
    ref_range_high: Option<f64>,
    flag: String,
}

#[derive(Debug, Serialize)]
struct SymptomDay {
    date: String,
    wellness_score: i64,
    daily_notes: String,
    symptoms: Vec<SymptomLogRow>,
}

#[derive(Debug, Serialize)]
struct SymptomLogRow {
    symptom_name: String,
    category: String,
    severity: i64,
    notes: String,
}

#[derive(Debug, Serialize)]
struct TrendPointOut {
    test_date: String,
    value: Option<f64>,
    text_value: String,
    flag: String,
}

#[derive(Debug, Serialize)]
struct DailySummaryOut {
    date: String,
    wellness_score: i64,
    notes: String,
}

#[derive(Clone)]
pub struct TrackerMcp {
    tool_router: ToolRouter<TrackerMcp>,
}

#[tool_router]
impl TrackerMcp {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }

    #[tool(description = "Get lab results from the last N days, grouped by session")]
    fn get_recent_labs(
        &self,
        Parameters(DaysParam { days }): Parameters<DaysParam>,
    ) -> Result<CallToolResult, McpError> {
        let conn = open_db()?;
        check_enabled(&conn)?;
        let mut stmt = conn.prepare(
            "SELECT id, test_date, lab_name, notes FROM lab_sessions
             WHERE test_date >= date('now', '-' || ?1 || ' days')
             ORDER BY test_date DESC",
        ).map_err(|e| McpError::internal_error(e.to_string(), None))?;

        let sessions: Vec<(i64, String, String, String)> = stmt.query_map(params![days], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
        }).map_err(|e| McpError::internal_error(e.to_string(), None))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        let mut output = Vec::new();
        for (sid, test_date, lab_name, notes) in sessions {
            let mut rstmt = conn.prepare(
                "SELECT test_name, panel, value, text_value, unit, ref_range_low, ref_range_high, flag
                 FROM lab_results WHERE session_id = ?1 ORDER BY panel, test_name",
            ).map_err(|e| McpError::internal_error(e.to_string(), None))?;

            let results: Vec<LabResultRow> = rstmt.query_map(params![sid], |row| {
                Ok(LabResultRow {
                    test_name: row.get(0)?,
                    panel: row.get(1)?,
                    value: row.get(2)?,
                    text_value: row.get(3)?,
                    unit: row.get(4)?,
                    ref_range_low: row.get(5)?,
                    ref_range_high: row.get(6)?,
                    flag: row.get(7)?,
                })
            }).map_err(|e| McpError::internal_error(e.to_string(), None))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

            output.push(LabSessionResult { test_date, lab_name, notes, results });
        }

        let json = serde_json::to_string_pretty(&output)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;
        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    #[tool(description = "Get all currently flagged abnormal lab values (latest result per test where flag is not normal)")]
    fn get_abnormal_labs(&self) -> Result<CallToolResult, McpError> {
        let conn = open_db()?;
        check_enabled(&conn)?;
        let mut stmt = conn.prepare(
            "SELECT r.test_name, r.panel, r.value, r.text_value, r.unit,
                    r.ref_range_low, r.ref_range_high, r.flag, s.test_date
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
        ).map_err(|e| McpError::internal_error(e.to_string(), None))?;

        #[derive(Serialize)]
        struct AbnormalResult {
            test_name: String,
            panel: String,
            value: Option<f64>,
            text_value: String,
            unit: String,
            ref_range_low: Option<f64>,
            ref_range_high: Option<f64>,
            flag: String,
            test_date: String,
        }

        let results: Vec<AbnormalResult> = stmt.query_map([], |row| {
            Ok(AbnormalResult {
                test_name: row.get(0)?,
                panel: row.get(1)?,
                value: row.get(2)?,
                text_value: row.get(3)?,
                unit: row.get(4)?,
                ref_range_low: row.get(5)?,
                ref_range_high: row.get(6)?,
                flag: row.get(7)?,
                test_date: row.get(8)?,
            })
        }).map_err(|e| McpError::internal_error(e.to_string(), None))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        let json = serde_json::to_string_pretty(&results)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;
        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    #[tool(description = "Get symptom logs with severity for the last N days")]
    fn get_symptom_history(
        &self,
        Parameters(DaysParam { days }): Parameters<DaysParam>,
    ) -> Result<CallToolResult, McpError> {
        let conn = open_db()?;
        check_enabled(&conn)?;
        let mut dstmt = conn.prepare(
            "SELECT DISTINCT log_date FROM symptom_logs
             WHERE log_date >= date('now', '-' || ?1 || ' days')
             ORDER BY log_date DESC",
        ).map_err(|e| McpError::internal_error(e.to_string(), None))?;

        let dates: Vec<String> = dstmt.query_map(params![days], |row| row.get(0))
            .map_err(|e| McpError::internal_error(e.to_string(), None))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        let mut output = Vec::new();
        for date in dates {
            let (wellness_score, daily_notes) = conn.query_row(
                "SELECT wellness_score, notes FROM daily_summaries WHERE log_date = ?1",
                params![date],
                |row| Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?)),
            ).unwrap_or((5, String::new()));

            let mut sstmt = conn.prepare(
                "SELECT s.name, s.category, sl.severity, sl.notes
                 FROM symptom_logs sl
                 JOIN symptoms s ON sl.symptom_id = s.id
                 WHERE sl.log_date = ?1
                 ORDER BY s.sort_order",
            ).map_err(|e| McpError::internal_error(e.to_string(), None))?;

            let symptoms: Vec<SymptomLogRow> = sstmt.query_map(params![date], |row| {
                Ok(SymptomLogRow {
                    symptom_name: row.get(0)?,
                    category: row.get(1)?,
                    severity: row.get(2)?,
                    notes: row.get(3)?,
                })
            }).map_err(|e| McpError::internal_error(e.to_string(), None))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

            output.push(SymptomDay { date, wellness_score, daily_notes, symptoms });
        }

        let json = serde_json::to_string_pretty(&output)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;
        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    #[tool(description = "Get time series data for a specific lab test over the last N days")]
    fn get_trends(
        &self,
        Parameters(TrendsParam { test_name, days }): Parameters<TrendsParam>,
    ) -> Result<CallToolResult, McpError> {
        let conn = open_db()?;
        check_enabled(&conn)?;
        let mut stmt = conn.prepare(
            "SELECT s.test_date, r.value, r.text_value, r.flag
             FROM lab_results r
             JOIN lab_sessions s ON r.session_id = s.id
             WHERE r.test_name = ?1
               AND s.test_date >= date('now', '-' || ?2 || ' days')
             ORDER BY s.test_date ASC",
        ).map_err(|e| McpError::internal_error(e.to_string(), None))?;

        let points: Vec<TrendPointOut> = stmt.query_map(params![test_name, days], |row| {
            Ok(TrendPointOut {
                test_date: row.get(0)?,
                value: row.get(1)?,
                text_value: row.get(2)?,
                flag: row.get(3)?,
            })
        }).map_err(|e| McpError::internal_error(e.to_string(), None))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        let json = serde_json::to_string_pretty(&points)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;
        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    #[tool(description = "Get daily wellness scores and notes for the last N days")]
    fn get_daily_summaries(
        &self,
        Parameters(DaysParam { days }): Parameters<DaysParam>,
    ) -> Result<CallToolResult, McpError> {
        let conn = open_db()?;
        check_enabled(&conn)?;
        let mut stmt = conn.prepare(
            "SELECT log_date, wellness_score, notes FROM daily_summaries
             WHERE log_date >= date('now', '-' || ?1 || ' days')
             ORDER BY log_date DESC",
        ).map_err(|e| McpError::internal_error(e.to_string(), None))?;

        let summaries: Vec<DailySummaryOut> = stmt.query_map(params![days], |row| {
            Ok(DailySummaryOut {
                date: row.get(0)?,
                wellness_score: row.get(1)?,
                notes: row.get(2)?,
            })
        }).map_err(|e| McpError::internal_error(e.to_string(), None))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        let json = serde_json::to_string_pretty(&summaries)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;
        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    #[tool(description = "Insert a new lab session with results. Use this to save lab results parsed from a lab report. \
        Each result needs at minimum a test_name and either a numeric value or text_value. \
        Set the flag to 'H' (high), 'L' (low), or 'A' (abnormal) if the result is outside the reference range, \
        or 'N' for normal. Results where both value and text_value are empty will be skipped. \
        Requires MCP write access to be enabled in the app settings.")]
    fn insert_lab_session(
        &self,
        Parameters(params): Parameters<InsertLabSessionParam>,
    ) -> Result<CallToolResult, McpError> {
        // Use read-only connection first to check settings
        let ro_conn = open_db()?;
        check_enabled(&ro_conn)?;
        check_write_enabled(&ro_conn)?;
        drop(ro_conn);

        let conn = open_db_rw()?;
        conn.execute_batch("PRAGMA foreign_keys=ON;")
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        conn.execute(
            "INSERT INTO lab_sessions (test_date, lab_name, notes) VALUES (?1, ?2, ?3)",
            rusqlite::params![
                params.test_date,
                params.lab_name.unwrap_or_default(),
                params.notes.unwrap_or_default(),
            ],
        ).map_err(|e| McpError::internal_error(format!("Failed to insert lab session: {}", e), None))?;

        let session_id = conn.last_insert_rowid();
        let mut inserted = 0u32;

        for r in &params.results {
            let text_val = r.text_value.clone().unwrap_or_default();
            if r.value.is_none() && text_val.is_empty() {
                continue;
            }
            conn.execute(
                "INSERT INTO lab_results (session_id, test_name, panel, value, text_value, unit, ref_range_low, ref_range_high, flag)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                rusqlite::params![
                    session_id,
                    r.test_name,
                    r.panel.clone().unwrap_or_default(),
                    r.value,
                    text_val,
                    r.unit.clone().unwrap_or_default(),
                    r.ref_range_low,
                    r.ref_range_high,
                    r.flag.clone().unwrap_or_else(|| "N".to_string()),
                ],
            ).map_err(|e| McpError::internal_error(format!("Failed to insert lab result: {}", e), None))?;
            inserted += 1;
        }

        let response = serde_json::json!({
            "session_id": session_id,
            "results_inserted": inserted,
        });
        Ok(CallToolResult::success(vec![Content::text(response.to_string())]))
    }
}

#[tool_handler]
impl ServerHandler for TrackerMcp {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .build(),
            server_info: Implementation::from_build_env(),
            instructions: Some(
                "MCP server for symptom and lab test tracking data. \
                 Read tools: retrieve recent labs, abnormal values, symptom history, \
                 trends for specific tests, and daily wellness summaries. \
                 Write tools: insert a new lab session with results (e.g. parsed from a pasted lab report). \
                 Write access must be enabled separately by the user in the app settings.".into(),
            ),
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    tracing::info!("Starting Symptom Test Tracker MCP server");

    let service = TrackerMcp::new()
        .serve(stdio())
        .await
        .inspect_err(|e| {
            tracing::error!("serving error: {:?}", e);
        })?;

    service.waiting().await?;
    Ok(())
}
