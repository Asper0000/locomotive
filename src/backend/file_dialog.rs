//! Native file and folder dialogs using platform-provided utilities.

use std::path::PathBuf;
use std::process::Command;

use super::steam::CREATE_NO_WINDOW;
use std::os::windows::process::CommandExt;

pub(crate) fn choose_files() -> Result<Vec<PathBuf>, String> {
    let script = r#"
$ErrorActionPreference = 'Stop'
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8
Add-Type -AssemblyName System.Windows.Forms
$dialog = New-Object System.Windows.Forms.OpenFileDialog
$dialog.Multiselect = $true
$dialog.CheckFileExists = $true
$dialog.Title = 'Import manifest and Lua files'
$dialog.Filter = 'Manifest and Lua files (*.manifest;*.manifests;*.lua)|*.manifest;*.manifests;*.lua|All files (*.*)|*.*'
if ($dialog.ShowDialog() -eq [System.Windows.Forms.DialogResult]::OK) {
    $dialog.FileNames | ForEach-Object { [Console]::WriteLine($_) }
}
"#;
    let mut command = Command::new("powershell.exe");
    command.args(["-NoProfile", "-NonInteractive", "-STA", "-Command", script]);
    command.creation_flags(CREATE_NO_WINDOW);
    let output = command
        .output()
        .map_err(|error| format!("Could not open the file picker: {error}"))?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_owned());
    }
    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(PathBuf::from)
        .collect())
}

pub(crate) fn choose_directory() -> Result<Option<PathBuf>, String> {
    let script = r#"
$ErrorActionPreference = 'Stop'
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8
Add-Type -AssemblyName System.Windows.Forms
$dialog = New-Object System.Windows.Forms.FolderBrowserDialog
$dialog.Description = 'Choose a local Locomotive library folder'
$dialog.ShowNewFolderButton = $true
if ($dialog.ShowDialog() -eq [System.Windows.Forms.DialogResult]::OK) {
    [Console]::WriteLine($dialog.SelectedPath)
}
"#;
    let mut command = Command::new("powershell.exe");
    command.args(["-NoProfile", "-NonInteractive", "-STA", "-Command", script]);
    command.creation_flags(CREATE_NO_WINDOW);
    let output = command
        .output()
        .map_err(|error| format!("Could not open the folder picker: {error}"))?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_owned());
    }
    let path = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    Ok((!path.is_empty()).then(|| PathBuf::from(path)))
}