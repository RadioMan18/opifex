use tauri::AppHandle;
use tauri_plugin_dialog::DialogExt;

/// Writes text to a file the user picks in a native save dialog.
///
/// The webview never supplies a path: it can only ask for a dialog, and the
/// dialog's own return value is what gets written. That removes the arbitrary
/// file-write primitive the previous path-taking command exposed to anything
/// that could reach the IPC layer.
#[tauri::command]
pub async fn save_text_file(
    app: AppHandle,
    default_name: String,
    contents: String,
) -> Result<Option<String>, String> {
    let picked = app
        .dialog()
        .file()
        .set_file_name(&default_name)
        .add_filter("JSON", &["json"])
        .blocking_save_file();

    let Some(file_path) = picked else {
        return Ok(None);
    };
    let path = file_path.into_path().map_err(|e| e.to_string())?;

    std::fs::write(&path, contents.as_bytes())
        .map_err(|e| format!("Failed to write '{}': {}", path.display(), e))?;

    Ok(Some(path.display().to_string()))
}

/// Reads a file the user picks in a native open dialog. Mirrors `save_text_file`:
/// no path arrives from the webview.
#[tauri::command]
pub async fn load_text_file(app: AppHandle) -> Result<Option<String>, String> {
    let picked = app.dialog().file().add_filter("JSON", &["json"]).blocking_pick_file();

    let Some(file_path) = picked else {
        return Ok(None);
    };
    let path = file_path.into_path().map_err(|e| e.to_string())?;

    let text = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read '{}': {}", path.display(), e))?;

    Ok(Some(text))
}
