/// File-system write command — called from the frontend after the native
/// Save dialog resolves a path.
#[tauri::command]
pub fn write_file(path: String, content: String) -> Result<(), String> {
    std::fs::write(&path, content.as_bytes())
        .map_err(|e| format!("Failed to write file '{}': {}", path, e))
}
