//! Validated import and removal operations for files in the local library.

use std::fs::{self, File};
use std::io::{BufReader, Read};
use std::path::Path;

use crate::state::{FileKind, PendingFile, library_state::{ImportOutcome, LibraryEntry, LibrarySnapshot}};

use super::{archive, library};

/// Copies staged inputs to the two local library folders and then rescans them.
pub(crate) fn import_files(
    manifest_dir: &Path,
    lua_dir: &Path,
    files: &[PendingFile],
) -> ImportOutcome {
    let settings = match archive::normalize_and_validate(manifest_dir.to_path_buf(), lua_dir.to_path_buf()) {
        Ok(settings) => settings,
        Err(error) => {
            return ImportOutcome {
                snapshot: failed_snapshot(error.clone()),
                imported: 0,
                skipped: files.len(),
                errors: vec![error],
            };
        }
    };

    let mut imported = 0;
    let mut skipped = 0;
    let mut errors = Vec::new();

    for file in files {
        let kind = FileKind::from_path(&file.path);
        if kind == FileKind::Unknown || kind != file.kind {
            skipped += 1;
            continue;
        }
        if !file.path.is_file() {
            skipped += 1;
            errors.push(format!("File no longer exists: {}", file.path.display()));
            continue;
        }
        let folder = match kind {
            FileKind::Manifest => &settings.manifest_dir,
            FileKind::Lua => &settings.lua_dir,
            FileKind::Unknown => continue,
        };
        if let Err(error) = fs::create_dir_all(folder) {
            skipped += 1;
            errors.push(format!("Could not create '{}': {error}", folder.display()));
            continue;
        }

        let Some(filename) = file.path.file_name() else {
            skipped += 1;
            errors.push(format!("File has no name: {}", file.path.display()));
            continue;
        };
        let destination = folder.join(filename);
        if same_file(&file.path, &destination) {
            skipped += 1;
            continue;
        }
        if destination.exists() {
            if same_contents(&file.path, &destination) {
                skipped += 1;
                continue;
            }
            skipped += 1;
            errors.push(format!(
                "A different file named '{}' already exists in '{}'. Rename one of the files before importing it.",
                filename.to_string_lossy(),
                folder.display()
            ));
            continue;
        }

        match fs::copy(&file.path, &destination) {
            Ok(_) => imported += 1,
            Err(error) => {
                skipped += 1;
                errors.push(format!(
                    "Could not copy '{}' to '{}': {error}",
                    file.path.display(),
                    destination.display()
                ));
            }
        }
    }

    ImportOutcome {
        snapshot: library::scan(&settings.manifest_dir, &settings.lua_dir),
        imported,
        skipped,
        errors,
    }
}

/// Moves any orphaned `.lua` files (tracked in `entry.source_warning`)
/// into the configured directory. Files are copied first, then removed
/// from the source; if the target already holds an identical file the
/// source is just deleted. If the target holds a *different* file with
/// the same name, the orphan is left in place and reported.
pub(crate) fn fix_orphan_entry(entry: &LibraryEntry) -> Result<usize, String> {
    let Some(warning) = &entry.source_warning else {
        return Err("This entry has no orphaned files to move.".into());
    };
    if warning.current_paths.is_empty() {
        return Ok(0);
    }
    fs::create_dir_all(&warning.configured_dir).map_err(|error| {
        format!(
            "Could not create '{}': {error}",
            warning.configured_dir.display()
        )
    })?;

    let mut moved = 0usize;
    let mut errors = Vec::new();
    for source in &warning.current_paths {
        if !source.is_file() {
            continue;
        }
        let Some(filename) = source.file_name() else { continue };
        let destination = warning.configured_dir.join(filename);
        if destination.exists() {
            if same_contents(source, &destination) {
                match fs::remove_file(source) {
                    Ok(()) => moved += 1,
                    Err(error) => errors.push(format!(
                        "Could not remove duplicate '{}': {error}",
                        source.display()
                    )),
                }
            } else {
                errors.push(format!(
                    "A different file named '{}' already exists in '{}'.",
                    filename.to_string_lossy(),
                    warning.configured_dir.display()
                ));
            }
            continue;
        }
        match fs::copy(source, &destination) {
            Ok(_) => {
                if let Err(error) = fs::remove_file(source) {
                    moved += 1;
                    errors.push(format!(
                        "Copied '{}' but could not remove the original: {error}",
                        source.display()
                    ));
                } else {
                    moved += 1;
                }
            }
            Err(error) => errors.push(format!(
                "Could not copy '{}': {error}",
                source.display()
            )),
        }
    }

    if errors.is_empty() {
        Ok(moved)
    } else {
        Err(format!(
            "Moved {moved} file(s), with errors: {}",
            errors.join("; ")
        ))
    }
}

/// Deletes only the selected entry's files from the configured local library folders.
///
/// Orphaned `.lua` files tracked in `entry.source_warning` are also
/// removed: the scanner found them, so they are safe to delete, and
/// removing an entry whose only lua is orphaned would otherwise leave
/// the user unable to clean up.
pub(crate) fn remove_entry(
    manifest_dir: &Path,
    lua_dir: &Path,
    entry: &LibraryEntry,
) -> Result<usize, String> {
    let settings = archive::normalize_and_validate(manifest_dir.to_path_buf(), lua_dir.to_path_buf())?;
    let mut removed = 0;
    let mut errors = Vec::new();

    for file in &entry.files {
        let expected_folder = match file.kind {
            FileKind::Manifest => &settings.manifest_dir,
            FileKind::Lua => &settings.lua_dir,
            FileKind::Unknown => continue,
        };
        if file.path.parent() != Some(expected_folder.as_path())
            || FileKind::from_path(&file.path) != file.kind
            || !file.path.is_file()
        {
            errors.push(format!("Refusing to remove a file outside the library: {}", file.path.display()));
            continue;
        }
        match fs::remove_file(&file.path) {
            Ok(()) => removed += 1,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => errors.push(format!("Could not remove '{}': {error}", file.path.display())),
        }
    }

    if let Some(warning) = &entry.source_warning {
        for path in &warning.current_paths {
            if !path.is_file() {
                continue;
            }
            match fs::remove_file(path) {
                Ok(()) => removed += 1,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => errors.push(format!(
                    "Could not remove orphaned '{}': {error}",
                    path.display()
                )),
            }
        }
    }

    if errors.is_empty() {
        Ok(removed)
    } else {
        Err(format!("Removed {removed} file(s), but some files could not be removed: {}", errors.join("; ")))
    }
}

fn same_file(source: &Path, destination: &Path) -> bool {
    source == destination
        || source
            .canonicalize()
            .ok()
            .zip(destination.canonicalize().ok())
            .is_some_and(|(left, right)| left == right)
}

fn same_contents(left: &Path, right: &Path) -> bool {
    let Ok(left_metadata) = fs::metadata(left) else {
        return false;
    };
    let Ok(right_metadata) = fs::metadata(right) else {
        return false;
    };
    if left_metadata.len() != right_metadata.len() {
        return false;
    }
    let Ok(left) = File::open(left) else { return false; };
    let Ok(right) = File::open(right) else { return false; };
    let mut left = BufReader::new(left);
    let mut right = BufReader::new(right);
    let mut left_buffer = [0_u8; 16 * 1024];
    let mut right_buffer = [0_u8; 16 * 1024];
    loop {
        let Ok(left_len) = left.read(&mut left_buffer) else { return false; };
        let Ok(right_len) = right.read(&mut right_buffer) else { return false; };
        if left_len != right_len || left_buffer[..left_len] != right_buffer[..right_len] {
            return false;
        }
        if left_len == 0 {
            return true;
        }
    }
}

fn failed_snapshot(error: String) -> LibrarySnapshot {
    LibrarySnapshot { entries: Vec::new(), error: Some(error) }
}