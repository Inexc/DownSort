pub mod category;
pub mod duplicates;
pub mod model;
pub mod scanner;
pub mod stats;
pub mod trash_ops;

pub use category::FileCategory;
pub use duplicates::{compute_file_sha256, find_and_mark_duplicates, group_duplicate_items, DuplicateGroup};
pub use model::{CategoryStats, FileItem, ScanSummary};
pub use scanner::{default_download_dir, format_bytes, format_system_time, is_file_old, open_file_in_system, scan_directory, ScanOptions};
pub use stats::compute_summary;
pub use trash_ops::{restore_from_trash, send_multiple_to_trash, send_to_trash, RestoreResult, TrashResult};

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::time::{Duration, SystemTime};

    #[test]
    fn test_category_from_extension() {
        // Документы: doc, docx, odt, rtf
        assert_eq!(FileCategory::from_extension("doc"), FileCategory::Documents);
        assert_eq!(FileCategory::from_extension(".DOCX"), FileCategory::Documents);
        assert_eq!(FileCategory::from_extension("odt"), FileCategory::Documents);
        assert_eq!(FileCategory::from_extension("rtf"), FileCategory::Documents);

        // PDF: pdf
        assert_eq!(FileCategory::from_extension("pdf"), FileCategory::Pdf);
        assert_eq!(FileCategory::from_extension("PDF"), FileCategory::Pdf);

        // Таблицы: xls, xlsx, csv, ods
        assert_eq!(FileCategory::from_extension("xls"), FileCategory::Spreadsheets);
        assert_eq!(FileCategory::from_extension("xlsx"), FileCategory::Spreadsheets);
        assert_eq!(FileCategory::from_extension("csv"), FileCategory::Spreadsheets);
        assert_eq!(FileCategory::from_extension("ods"), FileCategory::Spreadsheets);

        // Фото: jpg, jpeg, png, webp, heic, gif, bmp
        assert_eq!(FileCategory::from_extension("jpg"), FileCategory::Photos);
        assert_eq!(FileCategory::from_extension("jpeg"), FileCategory::Photos);
        assert_eq!(FileCategory::from_extension("png"), FileCategory::Photos);
        assert_eq!(FileCategory::from_extension("webp"), FileCategory::Photos);
        assert_eq!(FileCategory::from_extension("heic"), FileCategory::Photos);
        assert_eq!(FileCategory::from_extension("gif"), FileCategory::Photos);
        assert_eq!(FileCategory::from_extension("bmp"), FileCategory::Photos);

        // Архивы: zip, rar, 7z, tar, gz
        assert_eq!(FileCategory::from_extension("zip"), FileCategory::Archives);
        assert_eq!(FileCategory::from_extension("rar"), FileCategory::Archives);
        assert_eq!(FileCategory::from_extension("7z"), FileCategory::Archives);
        assert_eq!(FileCategory::from_extension("tar"), FileCategory::Archives);
        assert_eq!(FileCategory::from_extension("gz"), FileCategory::Archives);

        // Установщики: exe, msi, deb, rpm, AppImage, dmg
        assert_eq!(FileCategory::from_extension("exe"), FileCategory::Installers);
        assert_eq!(FileCategory::from_extension("msi"), FileCategory::Installers);
        assert_eq!(FileCategory::from_extension("deb"), FileCategory::Installers);
        assert_eq!(FileCategory::from_extension("rpm"), FileCategory::Installers);
        assert_eq!(FileCategory::from_extension("AppImage"), FileCategory::Installers);
        assert_eq!(FileCategory::from_extension("dmg"), FileCategory::Installers);

        // Видео/Аудио: mp4, mkv, avi, mov, mp3, wav, flac
        assert_eq!(FileCategory::from_extension("mp4"), FileCategory::Media);
        assert_eq!(FileCategory::from_extension("mkv"), FileCategory::Media);
        assert_eq!(FileCategory::from_extension("avi"), FileCategory::Media);
        assert_eq!(FileCategory::from_extension("mov"), FileCategory::Media);
        assert_eq!(FileCategory::from_extension("mp3"), FileCategory::Media);
        assert_eq!(FileCategory::from_extension("wav"), FileCategory::Media);
        assert_eq!(FileCategory::from_extension("flac"), FileCategory::Media);

        // Прочее: всё остальное
        assert_eq!(FileCategory::from_extension("rs"), FileCategory::Other);
        assert_eq!(FileCategory::from_extension("txt"), FileCategory::Other);
        assert_eq!(FileCategory::from_extension("iso"), FileCategory::Other);
        assert_eq!(FileCategory::from_extension("unknown"), FileCategory::Other);
    }

    #[test]
    fn test_duplicate_grouping_logic() {
        let base_time = SystemTime::UNIX_EPOCH + Duration::from_secs(1000000);
        let time_older = base_time;
        let time_newer1 = base_time + Duration::from_secs(3600);
        let time_newer2 = base_time + Duration::from_secs(7200);

        let mut files = vec![
            // Group 1: 3 files with same hash and size
            FileItem {
                id: 1,
                path: PathBuf::from("/downloads/copy1.zip"),
                name: "copy1.zip".to_string(),
                size: 2048,
                extension: Some("zip".to_string()),
                category: FileCategory::Archives,
                created: Some(time_newer1),
                last_accessed: None,
                is_old: false,
                hash: Some("hash_group_alpha".to_string()),
                duplicate_group_id: None,
                is_original: false,
                selected_for_deletion: false,
            },
            FileItem {
                id: 2,
                path: PathBuf::from("/downloads/original.zip"),
                name: "original.zip".to_string(),
                size: 2048,
                extension: Some("zip".to_string()),
                category: FileCategory::Archives,
                created: Some(time_older), // OLDEST!
                last_accessed: None,
                is_old: false,
                hash: Some("hash_group_alpha".to_string()),
                duplicate_group_id: None,
                is_original: false,
                selected_for_deletion: false,
            },
            FileItem {
                id: 3,
                path: PathBuf::from("/downloads/copy2.zip"),
                name: "copy2.zip".to_string(),
                size: 2048,
                extension: Some("zip".to_string()),
                category: FileCategory::Archives,
                created: Some(time_newer2),
                last_accessed: None,
                is_old: false,
                hash: Some("hash_group_alpha".to_string()),
                duplicate_group_id: None,
                is_original: false,
                selected_for_deletion: false,
            },
            // Unique file (different hash)
            FileItem {
                id: 4,
                path: PathBuf::from("/downloads/unique.pdf"),
                name: "unique.pdf".to_string(),
                size: 4096,
                extension: Some("pdf".to_string()),
                category: FileCategory::Pdf,
                created: Some(base_time),
                last_accessed: None,
                is_old: false,
                hash: Some("unique_hash_beta".to_string()),
                duplicate_group_id: None,
                is_original: false,
                selected_for_deletion: false,
            },
        ];

        let groups = group_duplicate_items(&mut files);

        assert_eq!(groups.len(), 1);
        let group = &groups[0];
        assert_eq!(group.hash, "hash_group_alpha");
        assert_eq!(group.file_size, 2048);
        assert_eq!(group.original_file_id, 2); // File #2 is oldest!
        assert_eq!(group.duplicate_file_ids.len(), 2);
        assert_eq!(group.wasted_size, 2048 * 2); // 2 redundant copies

        // Verify file item flags
        let file1 = files.iter().find(|f| f.id == 1).unwrap();
        assert!(!file1.is_original);
        assert!(file1.selected_for_deletion);
        assert_eq!(file1.duplicate_group_id, Some(group.group_id));

        let file2 = files.iter().find(|f| f.id == 2).unwrap();
        assert!(file2.is_original);
        assert!(!file2.selected_for_deletion);
        assert_eq!(file2.duplicate_group_id, Some(group.group_id));

        let file3 = files.iter().find(|f| f.id == 3).unwrap();
        assert!(!file3.is_original);
        assert!(file3.selected_for_deletion);
        assert_eq!(file3.duplicate_group_id, Some(group.group_id));

        let file4 = files.iter().find(|f| f.id == 4).unwrap();
        assert_eq!(file4.duplicate_group_id, None);
        assert!(!file4.is_original);
        assert!(!file4.selected_for_deletion);
    }

    #[test]
    fn test_is_file_old_calculation() {
        let now = SystemTime::UNIX_EPOCH + Duration::from_secs(100 * 24 * 3600); // Day 100

        // 95 days ago -> OLD
        let accessed_95_days_ago = now - Duration::from_secs(95 * 24 * 3600);
        assert!(is_file_old(Some(accessed_95_days_ago), now, 90));

        // 90 days ago exactly -> OLD
        let accessed_90_days_ago = now - Duration::from_secs(90 * 24 * 3600);
        assert!(is_file_old(Some(accessed_90_days_ago), now, 90));

        // 89 days ago -> NOT old
        let accessed_89_days_ago = now - Duration::from_secs(89 * 24 * 3600);
        assert!(!is_file_old(Some(accessed_89_days_ago), now, 90));

        // 10 days ago -> NOT old
        let accessed_10_days_ago = now - Duration::from_secs(10 * 24 * 3600);
        assert!(!is_file_old(Some(accessed_10_days_ago), now, 90));

        // None -> NOT old
        assert!(!is_file_old(None, now, 90));
    }

    #[test]
    fn test_category_size_arithmetic_and_summary() {
        let files = vec![
            FileItem::new(1, PathBuf::from("a.pdf"), 1000, None, None, false),
            FileItem::new(2, PathBuf::from("b.pdf"), 2500, None, None, true), // old
            FileItem::new(3, PathBuf::from("c.png"), 5000, None, None, false),
            FileItem::new(4, PathBuf::from("d.zip"), 10000, None, None, true), // old
            FileItem::new(5, PathBuf::from("e.txt"), 500, None, None, false),
        ];

        let summary = compute_summary(&files);

        assert_eq!(summary.total_files, 5);
        assert_eq!(summary.total_size, 1000 + 2500 + 5000 + 10000 + 500); // 19000 bytes

        // Old files: file 2 (2500) and file 4 (10000)
        assert_eq!(summary.old_files_count, 2);
        assert_eq!(summary.old_files_total_size, 12500);

        // Largest category should be Archives (10000 bytes)
        assert_eq!(summary.largest_category, Some(FileCategory::Archives));

        // Check category stats
        let pdf_stat = summary.category_stats.iter().find(|s| s.category == FileCategory::Pdf).unwrap();
        assert_eq!(pdf_stat.count, 2);
        assert_eq!(pdf_stat.total_size, 3500);

        let photos_stat = summary.category_stats.iter().find(|s| s.category == FileCategory::Photos).unwrap();
        assert_eq!(photos_stat.count, 1);
        assert_eq!(photos_stat.total_size, 5000);

        let other_stat = summary.category_stats.iter().find(|s| s.category == FileCategory::Other).unwrap();
        assert_eq!(other_stat.count, 1);
        assert_eq!(other_stat.total_size, 500);
    }

    #[test]
    fn test_trash_and_restore() {
        let test_path = std::env::current_dir().unwrap().join("test_trash_sample.tmp");
        std::fs::write(&test_path, b"downsort trash test").unwrap();
        assert!(test_path.exists());

        // Send to trash
        let trash_res = send_to_trash(&test_path);
        if trash_res.is_err() {
            // Some CI or virtual environments might not have Recycle Bin enabled
            let _ = std::fs::remove_file(&test_path);
            return;
        }
        assert!(!test_path.exists());

        // Restore from trash
        let restore_res = restore_from_trash(&[test_path.clone()]);
        assert!(restore_res.is_ok());
        let res = restore_res.unwrap();
        assert!(res.restored.contains(&test_path));
        assert!(test_path.exists());

        // Clean up
        let _ = std::fs::remove_file(&test_path);
    }
}
