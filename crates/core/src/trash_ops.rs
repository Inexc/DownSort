use std::path::{Path, PathBuf};

#[derive(Debug, Default)]
pub struct TrashResult {
    pub deleted: Vec<PathBuf>,
    pub failed: Vec<(PathBuf, String)>,
}

/// Send a single file to the system trash.
pub fn send_to_trash(path: &Path) -> Result<(), trash::Error> {
    trash::delete(path)
}

/// Send multiple files to the system trash using batched native API.
pub fn send_multiple_to_trash(paths: &[PathBuf]) -> TrashResult {
    let mut result = TrashResult::default();

    if paths.is_empty() {
        return result;
    }

    match trash::delete_all(paths) {
        Ok(()) => {
            result.deleted = paths.to_vec();
        }
        Err(_) => {
            // Fallback to one-by-one if batch encountered a partial issue
            for path in paths {
                match trash::delete(path) {
                    Ok(()) => {
                        result.deleted.push(path.clone());
                    }
                    Err(e) => {
                        result.failed.push((path.clone(), e.to_string()));
                    }
                }
            }
        }
    }

    result
}

#[derive(Debug, Default)]
pub struct RestoreResult {
    pub restored: Vec<PathBuf>,
    pub failed: Vec<(PathBuf, String)>,
}

/// Restore files from system trash by their original paths.
pub fn restore_from_trash(paths: &[PathBuf]) -> Result<RestoreResult, String> {
    let mut result = RestoreResult::default();
    if paths.is_empty() {
        return Ok(result);
    }

    let trash_items = trash::os_limited::list().map_err(|e| e.to_string())?;
    let path_set: std::collections::HashSet<PathBuf> = paths.iter().cloned().collect();

    let to_restore: Vec<trash::TrashItem> = trash_items
        .into_iter()
        .filter(|item| path_set.contains(&item.original_path()))
        .collect();

    for item in to_restore {
        let orig = item.original_path();
        match trash::os_limited::restore_all(vec![item]) {
            Ok(()) => {
                result.restored.push(orig);
            }
            Err(e) => {
                result.failed.push((orig, e.to_string()));
            }
        }
    }

    Ok(result)
}
