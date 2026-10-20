use crate::commands::{
    create_agent, delete_agent, get_agent, list_agents, ollama_chat, ollama_chat_stream,
    ollama_get_default_model, ollama_list_models, ollama_pull_model, ollama_set_default_model,
    seed_starter_agents, update_agent,
    create_conversation, get_conversation, list_conversations, load_messages, save_message,
    export_agent, import_agent, rename_agent, duplicate_agent,
    save_plan_version, list_plan_versions, restore_plan_version,
    ollama_get_provider, ollama_set_provider, ollama_get_api_key, ollama_set_api_key,
    write_file, read_file,
};
use crate::db::init_db;
use std::path::PathBuf;
use tauri::Manager;

mod commands;
mod db;

fn app_data_dir() -> Result<PathBuf, String> {
    if cfg!(target_os = "windows") {
        std::env::var("APPDATA")
            .map(|p| PathBuf::from(p).join("Opifex"))
            .map_err(|_| "APPDATA environment variable not set".to_string())
    } else if cfg!(target_os = "macos") {
        std::env::var("HOME")
            .map(|p| PathBuf::from(p).join("Library/Application Support/Opifex"))
            .map_err(|_| "HOME environment variable not set".to_string())
    } else {
        std::env::var("HOME")
            .map(|p| PathBuf::from(p).join(".local/share/Opifex"))
            .map_err(|_| "HOME environment variable not set".to_string())
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .plugin(tauri_plugin_log::Builder::new().build())
    .plugin(tauri_plugin_dialog::init())
    .plugin(tauri_plugin_fs::init())
    .setup(|app| {
      let data_dir = app_data_dir().expect("Failed to resolve app data directory");
      let db_path = data_dir.join("opifex.db");
      let db_state = init_db(&db_path).expect("Failed to initialize database");
      app.manage(db_state);
      Ok(())
    })
    .invoke_handler(tauri::generate_handler![
      create_agent,
      get_agent,
      list_agents,
      update_agent,
      delete_agent,
      seed_starter_agents,
      ollama_list_models,
      ollama_pull_model,
      ollama_chat,
      ollama_chat_stream,
      ollama_get_default_model,
      ollama_set_default_model,
      ollama_get_provider,
      ollama_set_provider,
      ollama_get_api_key,
      ollama_set_api_key,
      create_conversation,
      get_conversation,
      list_conversations,
      load_messages,
      save_message,
      export_agent,
      import_agent,
      rename_agent,
      duplicate_agent,
      save_plan_version,
      list_plan_versions,
      restore_plan_version,
      write_file,
      read_file,
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
