use crate::dto::AppResponse;

#[tauri::command]
pub fn pick_folder_path() -> AppResponse<Option<String>> {
    match platform_pick_folder() {
        Ok(path) => AppResponse::success(path),
        Err(error) => {
            AppResponse::error("dialog_folder_failed", &error, "errors.dialogFolderFailed")
        }
    }
}

#[tauri::command]
pub fn pick_file_path(kind: String) -> AppResponse<Option<String>> {
    match platform_pick_file(&kind) {
        Ok(path) => AppResponse::success(path),
        Err(error) => AppResponse::error("dialog_file_failed", &error, "errors.dialogFileFailed"),
    }
}

#[cfg(windows)]
fn platform_pick_folder() -> Result<Option<String>, String> {
    run_powershell_dialog(
        r#"
Add-Type -AssemblyName System.Windows.Forms
$dialog = New-Object System.Windows.Forms.FolderBrowserDialog
$dialog.Description = 'Select a folder'
$dialog.ShowNewFolderButton = $true
if ($dialog.ShowDialog() -eq [System.Windows.Forms.DialogResult]::OK) {
  [Console]::OutputEncoding = [System.Text.Encoding]::UTF8
  [Console]::Write($dialog.SelectedPath)
}
"#,
    )
}

#[cfg(windows)]
fn platform_pick_file(kind: &str) -> Result<Option<String>, String> {
    let filter = match kind {
        "markdown" => "Markdown files (*.md)|*.md|All files (*.*)|*.*",
        "json" => "JSON files (*.json)|*.json|All files (*.*)|*.*",
        _ => "All files (*.*)|*.*",
    };
    let script = format!(
        r#"
Add-Type -AssemblyName System.Windows.Forms
$dialog = New-Object System.Windows.Forms.OpenFileDialog
$dialog.Filter = '{}'
$dialog.Multiselect = $false
if ($dialog.ShowDialog() -eq [System.Windows.Forms.DialogResult]::OK) {{
  [Console]::OutputEncoding = [System.Text.Encoding]::UTF8
  [Console]::Write($dialog.FileName)
}}
"#,
        filter
    );

    run_powershell_dialog(&script)
}

#[cfg(windows)]
fn run_powershell_dialog(script: &str) -> Result<Option<String>, String> {
    use std::os::windows::process::CommandExt;
    use std::process::Command;

    const CREATE_NO_WINDOW: u32 = 0x08000000;

    let output = Command::new("powershell")
        .args(["-NoProfile", "-STA", "-Command", script])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map_err(|error| format!("Failed to open Windows dialog: {error}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(if stderr.is_empty() {
            "Windows dialog command failed.".to_string()
        } else {
            stderr
        });
    }

    let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if path.is_empty() {
        Ok(None)
    } else {
        Ok(Some(path))
    }
}

#[cfg(not(windows))]
fn platform_pick_folder() -> Result<Option<String>, String> {
    Err("Native path picker is only implemented for Windows in V1.".to_string())
}

#[cfg(not(windows))]
fn platform_pick_file(_kind: &str) -> Result<Option<String>, String> {
    Err("Native path picker is only implemented for Windows in V1.".to_string())
}
