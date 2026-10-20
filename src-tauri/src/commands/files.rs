/// File-system write command — called from the frontend after the native
/// Save dialog resolves a path.
#[tauri::command]
pub fn write_file(path: String, content: String) -> Result<(), String> {
    std::fs::write(&path, content.as_bytes())
        .map_err(|e| format!("Failed to write file '{}': {}", path, e))
}

/// File-system read command — called from the frontend after the native
/// Open dialog resolves a path.
#[tauri::command]
pub fn read_file(path: String) -> Result<String, String> {
    std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read file '{}': {}", path, e))
}
