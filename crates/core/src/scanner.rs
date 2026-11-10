use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use walkdir::WalkDir;

use crate::model::FileItem;

#[derive(Debug, Clone)]
pub struct ScanOptions {
    pub root_path: PathBuf,
    pub max_depth: usize,
    pub old_threshold_days: u64,
}

impl Default for ScanOptions {
    fn default() -> Self {
        Self {
            root_path: default_download_dir(),
            max_depth: 2,
            old_threshold_days: 90,
        }
    }
}

/// Returns the user's Downloads directory, or falls back to home/current directory.
pub fn default_download_dir() -> PathBuf {
    if let Some(download_dir) = dirs::download_dir() {
        if download_dir.exists() {
            return download_dir;
        }
    }
    if let Some(home) = dirs::home_dir() {
        let dl = home.join("Downloads");
        if dl.exists() {
            return dl;
        }
        return home;
    }
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

/// Check if a file is considered "old" based on its last access/modified time.
pub fn is_file_old(last_accessed: Option<SystemTime>, now: SystemTime, threshold_days: u64) -> bool {
    if let Some(access_time) = last_accessed {
        let threshold_secs = threshold_days.saturating_mul(24 * 3600);
        if let Ok(duration) = now.duration_since(access_time) {
            return duration.as_secs() >= threshold_secs;
        }
    }
    false
}

/// Scan the directory recursively according to options.
pub fn scan_directory(options: &ScanOptions) -> Result<Vec<FileItem>, std::io::Error> {
    let now = SystemTime::now();
    let mut items = Vec::new();
    let mut current_id = 0;

    let walker = WalkDir::new(&options.root_path)
        .max_depth(options.max_depth)
        .follow_links(false)
        .into_iter();

    for entry_res in walker {
        let entry = match entry_res {
            Ok(e) => e,
            Err(_) => continue, // Skip inaccessible entries gracefully
        };

        let file_type = entry.file_type();
        if !file_type.is_file() {
            continue;
        }

        let path = entry.path().to_path_buf();
        let metadata = match fs::metadata(&path) {
            Ok(m) => m,
            Err(_) => continue,
        };

        let size = metadata.len();
        let created = metadata.created().ok();

        // On many modern filesystems, access time might be disabled or fallback to modified.
        // We prefer accessed(), and fallback to modified() if accessed is unavailable.
        let last_accessed = metadata.accessed().ok().or_else(|| metadata.modified().ok());

        let is_old = is_file_old(last_accessed, now, options.old_threshold_days);

        current_id += 1;
        items.push(FileItem::new(
            current_id,
            path,
            size,
            created,
            last_accessed,
            is_old,
        ));
    }

    Ok(items)
}

/// Format bytes into a human-readable string (B, KB, MB, GB, TB).
pub fn format_bytes(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;
    const TB: f64 = GB * 1024.0;

    let b = bytes as f64;
    if b >= TB {
        format!("{:.2} ТБ", b / TB)
    } else if b >= GB {
        format!("{:.2} ГБ", b / GB)
    } else if b >= MB {
        format!("{:.1} МБ", b / MB)
    } else if b >= KB {
        format!("{:.1} КБ", b / KB)
    } else {
        format!("{} Б", bytes)
    }
}

/// Format SystemTime into a simple date string "YYYY-MM-DD"
pub fn format_system_time(time: Option<SystemTime>) -> String {
    let Some(t) = time else {
        return "—".to_string();
    };

    let duration = match t.duration_since(UNIX_EPOCH) {
        Ok(d) => d,
        Err(_) => return "—".to_string(),
    };

    let total_secs = duration.as_secs();
    // Rough calendar date calculation without external heavy chrono dependencies
    let days_since_epoch = total_secs / 86400;

    // Convert days since Jan 1 1970 to year, month, day
    let (year, month, day) = days_to_ymd(days_since_epoch);
    format!("{:04}-{:02}-{:02}", year, month, day)
}

fn days_to_ymd(days: u64) -> (u32, u32, u32) {
    // Gregorian calendar calculation
    let z = (days as i64) + 719468;
    let era = z / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = mp + (if mp < 10 { 3 } else { -9 });
    let year = y + (if m <= 2 { 1 } else { 0 });
    (year as u32, m as u32, d as u32)
}

/// Open a file in the default system application (e.g. PDF reader, image viewer, text editor)
pub fn open_file_in_system(path: &Path) -> std::io::Result<()> {
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        let mut cmd = std::process::Command::new("cmd");
        cmd.args(["/c", "start", "", &path.to_string_lossy()]);
        cmd.creation_flags(CREATE_NO_WINDOW);
        cmd.spawn()?;
        Ok(())
    }
    #[cfg(not(target_os = "windows"))]
    {
        std::process::Command::new("xdg-open")
            .arg(path)
            .spawn()?;
        Ok(())
    }
}
