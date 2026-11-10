use std::path::PathBuf;
use std::time::SystemTime;
use crate::category::FileCategory;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileItem {
    pub id: usize,
    pub path: PathBuf,
    pub name: String,
    pub size: u64,
    pub extension: Option<String>,
    pub category: FileCategory,
    pub created: Option<SystemTime>,
    pub last_accessed: Option<SystemTime>,
    pub is_old: bool,
    pub hash: Option<String>,
    pub duplicate_group_id: Option<usize>,
    pub is_original: bool,
    pub selected_for_deletion: bool,
}

impl FileItem {
    pub fn new(
        id: usize,
        path: PathBuf,
        size: u64,
        created: Option<SystemTime>,
        last_accessed: Option<SystemTime>,
        is_old: bool,
    ) -> Self {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        let extension = path
            .extension()
            .map(|ext| ext.to_string_lossy().to_string());

        let category = match &extension {
            Some(ext) => FileCategory::from_extension(ext),
            None => FileCategory::Other,
        };

        Self {
            id,
            path,
            name,
            size,
            extension,
            category,
            created,
            last_accessed,
            is_old,
            hash: None,
            duplicate_group_id: None,
            is_original: false,
            selected_for_deletion: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CategoryStats {
    pub category: FileCategory,
    pub count: usize,
    pub total_size: u64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ScanSummary {
    pub total_files: usize,
    pub total_size: u64,
    pub category_stats: Vec<CategoryStats>,
    pub largest_category: Option<FileCategory>,
    pub old_files_count: usize,
    pub old_files_total_size: u64,
    pub duplicate_groups_count: usize,
    pub duplicate_files_count: usize,
    pub duplicate_wasted_size: u64,
}
