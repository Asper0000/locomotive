use std::path::{Path, PathBuf};
use iced::futures::{SinkExt, StreamExt};
use iced::stream;
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

/// Fetches the latest RustyTools release tag from the public GitHub API.
/// Returns `None` when curl is unavailable, the request fails, or the
/// response does not contain a recognizable version string.
pub fn fetch_latest_version() -> Option<String> {
    #[cfg(target_os = "windows")]
    let mut cmd = std::process::Command::new("curl.exe");
    #[cfg(not(target_os = "windows"))]
    let mut cmd = std::process::Command::new("curl");
    cmd.args([
        "--fail",
        "--location",
        "--silent",
        "--show-error",
        "--connect-timeout", "3",
        "--max-time", "6",
        "--header", "User-Agent: Locomotive/0.0.1",
        "https://api.github.com/repos/Asper0000/rusty-tools/releases/latest",
    ]);
    #[cfg(target_os = "windows")]
    cmd.creation_flags(crate::backend::steam::CREATE_NO_WINDOW);
    let output = cmd.output().ok().filter(|o| o.status.success())?;
    let body = String::from_utf8(output.stdout).ok()?;
    if let Some(tag) = super::artwork_json::json_string_for_key(&body, "tag_name") {
        let tag = tag.trim().to_owned();
        if !tag.is_empty() {
            return Some(tag);
        }
    }
    // Fallback: release titles look like "v0.0.2 RustyTools". Take the
    // first whitespace-delimited token when it looks like a version.
    super::artwork_json::json_string_for_key(&body, "name").and_then(|name| {
        let name = name.trim();
        let first = name.split_whitespace().next()?;
        if first.starts_with('v') || first.chars().next().is_some_and(|c| c.is_ascii_digit()) {
            Some(first.to_owned())
        } else {
            None
        }
    })
}

pub fn is_installed(steam_root: &Path) -> bool {
    steam_root.join("RustyTools.dll").exists()
}

pub fn is_steam_running() -> bool {
    #[cfg(target_os = "windows")]
    {
        let mut cmd = std::process::Command::new("tasklist");
        cmd.creation_flags(crate::backend::steam::CREATE_NO_WINDOW);
        cmd.args(["/FI", "IMAGENAME eq steam.exe", "/NH"]);
        if let Ok(out) = cmd.output() {
            let text = String::from_utf8_lossy(&out.stdout);
            return text.contains("steam.exe");
        }
        false
    }
    #[cfg(not(target_os = "windows"))]
    false
}

pub fn restart_steam() {
    #[cfg(target_os = "windows")]
    {
        let mut kill = std::process::Command::new("taskkill");
        kill.creation_flags(crate::backend::steam::CREATE_NO_WINDOW);
        let _ = kill.args(["/F", "/IM", "steam.exe"]).status();
        
        std::thread::sleep(std::time::Duration::from_secs(2));

        let mut start = std::process::Command::new("cmd");
        start.creation_flags(crate::backend::steam::CREATE_NO_WINDOW);
        let _ = start.args(["/C", "start steam://open/main"]).status();
    }
}

pub fn download_and_install(steam_root: PathBuf) -> impl iced::futures::Stream<Item = crate::message::Message> {
    stream::channel(100, move |mut output: iced::futures::channel::mpsc::Sender<crate::message::Message>| async move {
        let (tx, mut rx) = iced::futures::channel::mpsc::channel(100);
        let root = steam_root.clone();

        std::thread::spawn(move || {
            let mut tx = tx;
            let _ = tx.try_send(crate::message::Message::SetupProgress(0.03));
            let _ = tx.try_send(crate::message::Message::SetupStatus("Preparing installation".into()));
            std::thread::sleep(std::time::Duration::from_millis(150));

            let _ = tx.try_send(crate::message::Message::SetupProgress(0.08));
            let _ = tx.try_send(crate::message::Message::SetupStatus("Downloading release from GitHub".into()));

            #[cfg(target_os = "windows")]
            {
                let temp_dir = std::env::temp_dir().join("locomotive_setup");
                let _ = std::fs::create_dir_all(&temp_dir);
                let zip_path = temp_dir.join("release.zip");

                let url = "https://github.com/Asper0000/rusty-tools/releases/latest/download/release.zip";

                // 1. Download
                let mut cmd = std::process::Command::new("curl.exe");
                cmd.creation_flags(crate::backend::steam::CREATE_NO_WINDOW);
                let output = cmd.args(["-L", "-s", "-o", zip_path.to_str().unwrap(), url]).output();

                match output {
                    Ok(out) if !out.status.success() => {
                        let err = String::from_utf8_lossy(&out.stderr);
                        let stdout = String::from_utf8_lossy(&out.stdout);
                        let _ = tx.try_send(crate::message::Message::SetupFinished(Err(format!("Download failed. Status: {}. Err: {}. Out: {}", out.status, err, stdout))));
                        return;
                    }
                    Err(e) => {
                        let _ = tx.try_send(crate::message::Message::SetupFinished(Err(format!("Failed to run curl: {}", e))));
                        return;
                    }
                    _ => {}
                }

                let size_kb = std::fs::metadata(&zip_path).map(|m| m.len() / 1024).unwrap_or(0);
                let _ = tx.try_send(crate::message::Message::SetupProgress(0.35));
                let _ = tx.try_send(crate::message::Message::SetupStatus(format!("Downloaded {size_kb} KB  verifying archive")));

                // 1.5 Verify it's actually a ZIP (magic bytes 'PK')
                if let Ok(bytes) = std::fs::read(&zip_path) {
                    if bytes.len() < 2 || &bytes[0..2] != b"PK" {
                        let size = bytes.len();
                        let preview = String::from_utf8_lossy(&bytes[0..std::cmp::min(100, size)]).replace("\n", " ");
                        let _ = tx.try_send(crate::message::Message::SetupFinished(Err(format!("Downloaded file is not a valid zip (Size: {}). It might be a 404 page. Preview: {}", size, preview))));
                        return;
                    }
                } else {
                    let _ = tx.try_send(crate::message::Message::SetupFinished(Err("Could not read downloaded zip file.".into())));
                    return;
                }

                let _ = tx.try_send(crate::message::Message::SetupProgress(0.45));
                let _ = tx.try_send(crate::message::Message::SetupStatus("Extracting archive and copying DLLs".into()));
                std::thread::sleep(std::time::Duration::from_millis(150));

                // 2. Extract AND copy DLLs using PowerShell
                let ps_script = format!(
                    "Expand-Archive -Path '{}' -DestinationPath '{}' -Force; Get-ChildItem -Path '{}' -Filter *.dll -Recurse | Copy-Item -Destination '{}' -Force",
                    zip_path.display(), temp_dir.display(), temp_dir.display(), root.display()
                );

                let mut ps = std::process::Command::new("powershell.exe");
                ps.creation_flags(crate::backend::steam::CREATE_NO_WINDOW);
                ps.args(["-NoProfile", "-Command", &ps_script]);
                let output = ps.output();

                match output {
                    Ok(out) if !out.status.success() => {
                        let err = String::from_utf8_lossy(&out.stderr);
                        let _ = tx.try_send(crate::message::Message::SetupFinished(Err(format!("Extraction failed: {}", err))));
                        return;
                    }
                    Err(e) => {
                        let _ = tx.try_send(crate::message::Message::SetupFinished(Err(format!("Failed to run PowerShell: {}", e))));
                        return;
                    }
                    _ => {}
                }

                let _ = tx.try_send(crate::message::Message::SetupProgress(0.70));
                let _ = tx.try_send(crate::message::Message::SetupStatus("Verifying RustyTools.dll".into()));

                // 3. Verify it actually copied
                if !root.join("RustyTools.dll").exists() {
                    let _ = tx.try_send(crate::message::Message::SetupFinished(Err("PowerShell ran, but RustyTools.dll was not found in the destination folder. The zip might not contain it.".into())));
                    return;
                }

                let _ = tx.try_send(crate::message::Message::SetupProgress(0.88));
                let _ = tx.try_send(crate::message::Message::SetupStatus("Cleaning up temporary files".into()));
                let _ = std::fs::remove_dir_all(&temp_dir);
                std::thread::sleep(std::time::Duration::from_millis(200));

                let _ = tx.try_send(crate::message::Message::SetupProgress(1.0));
                let _ = tx.try_send(crate::message::Message::SetupStatus("Installation complete.".into()));
                let _ = tx.try_send(crate::message::Message::SetupFinished(Ok(())));
            }
        });

        while let Some(msg) = rx.next().await {
            let _ = output.send(msg).await;
        }
    })
}
pub fn remove_rustytools(steam_root: &Path) -> Result<(), String> {
    let dlls = ["RustyTools.dll", "dwmapi.dll", "xinput1_4.dll"];
    let mut failed = false;
    for dll in dlls {
        let path = steam_root.join(dll);
        if path.exists() {
            if std::fs::remove_file(&path).is_err() {
                failed = true;
            }
        }
    }
    if failed {
        Err("Could not delete some DLLs. Make sure Steam is completely closed.".into())
    } else {
        Ok(())
    }
}