mod commands;
mod db;

use db::Database;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let database = Database::new().expect("Failed to initialize database");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(database)
        .invoke_handler(tauri::generate_handler![
            commands::labs::get_custom_lab_tests,
            commands::labs::save_custom_lab_test,
            commands::labs::delete_custom_lab_test,
            commands::labs::get_lab_sessions,
            commands::labs::get_lab_session,
            commands::labs::save_lab_session,
            commands::labs::delete_lab_session,
            commands::labs::get_latest_abnormal,
            commands::labs::get_latest_abnormal_with_previous,
            commands::labs::get_trends,
            commands::labs::get_all_test_names,
            commands::symptoms::get_wellness_trends,
            commands::symptoms::get_symptom_trends,
            commands::symptoms::get_active_symptom_names,
            commands::symptoms::get_symptoms,
            commands::symptoms::save_symptom,
            commands::symptoms::delete_symptom,
            commands::symptoms::reorder_symptoms,
            commands::symptoms::get_symptom_log,
            commands::symptoms::save_symptom_log,
            commands::diagnoses::get_diagnoses,
            commands::diagnoses::save_diagnosis,
            commands::diagnoses::delete_diagnosis,
            commands::export::export_data,
            commands::settings::get_setting,
            commands::settings::set_setting,
            commands::artifacts::get_artifacts,
            commands::artifacts::get_artifact,
            commands::artifacts::save_artifact_paste,
            commands::artifacts::save_artifact_file,
            commands::artifacts::delete_artifact,
            commands::artifacts::extract_labs_from_artifact,
            commands::chat::get_chat_history,
            commands::chat::clear_chat_history,
            commands::chat::get_memory,
            commands::chat::save_memory,
            commands::chat::send_chat_message,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
