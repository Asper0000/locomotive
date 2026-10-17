//! Native file and folder dialogs using platform-provided utilities.

use std::path::PathBuf;
use std::process::Command;

use super::steam::CREATE_NO_WINDOW;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

/// Opens a multi-select file chooser filtered to supported import types.
pub(crate) fn choose_files() -> Result<Vec<PathBuf>, String> {
    #[cfg(target_os = "windows")]
    {
        return windows_choose_files();
    }
    #[cfg(target_os = "linux")]
    {
        return linux_choose_files();
    }
    #[allow(unreachable_code)]
    Err("File selection is not supported on this operating system.".into())
}

/// Opens a native folder chooser.
pub(crate) fn choose_directory() -> Result<Option<PathBuf>, String> {
    #[cfg(target_os = "windows")]
    {
        return windows_choose_directory();
    }
    #[cfg(target_os = "linux")]
    {
        return linux_choose_directory();
    }
    #[allow(unreachable_code)]
    Err("Folder selection is not supported on this operating system.".into())
}

#[cfg(target_os = "windows")]
fn windows_choose_files() -> Result<Vec<PathBuf>, String> {
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

#[cfg(target_os = "windows")]
fn windows_choose_directory() -> Result<Option<PathBuf>, String> {
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

#[cfg(target_os = "linux")]
fn linux_choose_files() -> Result<Vec<PathBuf>, String> {
    let zenity = Command::new("zenity")
        .args([
            "--file-selection",
            "--multiple",
            "--separator=\n",
            "--title=Import manifest and Lua files",
            "--file-filter=Manifest and Lua files | *.manifest *.manifests *.lua",
            "--file-filter=All files | *",
        ])
        .output();

    match zenity {
        Ok(output) if output.status.success() => {
            return Ok(String::from_utf8_lossy(&output.stdout)
                .lines()
                .map(str::trim)
                .filter(|path| !path.is_empty())
                .map(PathBuf::from)
                .collect());
        }
        Ok(output) if output.status.code() == Some(1) => return Ok(Vec::new()),
        Ok(output) => {
            let details = String::from_utf8_lossy(&output.stderr).trim().to_owned();
            if !details.is_empty() {
                return Err(format!("The file picker failed: {details}"));
            }
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(format!("Could not open the file picker: {error}")),
    }

    let output = Command::new("kdialog")
        .args([
            "--getopenfilename",
            ".",
            "Manifest/Lua files (*.manifest *.manifests *.lua)",
            "--multiple",
            "--separate-output",
        ])
        .output()
        .map_err(|_| {
            "No native file-picker utility was found. Install zenity or kdialog, or drag files into Locomotive.".to_owned()
        })?;
    if !output.status.success() {
        return Ok(Vec::new());
    }
    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::trim)
        .filter(|path| !path.is_empty())
        .map(PathBuf::from)
        .collect())
}

#[cfg(target_os = "linux")]
fn linux_choose_directory() -> Result<Option<PathBuf>, String> {
    let zenity = Command::new("zenity")
        .args([
            "--file-selection",
            "--directory",
            "--title=Choose a local Locomotive library folder",
        ])
        .output();
    match zenity {
        Ok(output) if output.status.success() => {
            let path = String::from_utf8_lossy(&output.stdout).trim().to_owned();
            return Ok((!path.is_empty()).then(|| PathBuf::from(path)));
        }
        Ok(output) if output.status.code() == Some(1) => return Ok(None),
        Ok(_) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(format!("Could not open the folder picker: {error}")),
    }

    let output = Command::new("kdialog")
        .args(["--getexistingdirectory", ".", "Choose a local library folder"])
        .output()
        .map_err(|_| {
            "No native folder-picker utility was found. Install zenity or kdialog, or type the folder path directly.".to_owned()
        })?;
    if !output.status.success() {
        return Ok(None);
    }
    let path = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    Ok((!path.is_empty()).then(|| PathBuf::from(path)))
}