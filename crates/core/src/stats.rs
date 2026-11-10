use crate::category::FileCategory;
use crate::model::{CategoryStats, FileItem, ScanSummary};

/// Compute complete scan summary from a slice of FileItems.
pub fn compute_summary(files: &[FileItem]) -> ScanSummary {
    let mut total_size: u64 = 0;
    let mut category_counts = [0usize; 8];
    let mut category_sizes = [0u64; 8];

    let mut old_files_count: usize = 0;
    let mut old_files_total_size: u64 = 0;

    let mut duplicate_files_count: usize = 0;
    let mut duplicate_wasted_size: u64 = 0;
    let mut max_duplicate_group_id: usize = 0;

    let all_categories = FileCategory::all();

    for file in files {
        total_size = total_size.saturating_add(file.size);

        let cat_idx = match file.category {
            FileCategory::Documents => 0,
            FileCategory::Pdf => 1,
            FileCategory::Spreadsheets => 2,
            FileCategory::Photos => 3,
            FileCategory::Archives => 4,
            FileCategory::Installers => 5,
            FileCategory::Media => 6,
            FileCategory::Other => 7,
        };

        category_counts[cat_idx] += 1;
        category_sizes[cat_idx] = category_sizes[cat_idx].saturating_add(file.size);

        if file.is_old {
            old_files_count += 1;
            old_files_total_size = old_files_total_size.saturating_add(file.size);
        }

        if let Some(group_id) = file.duplicate_group_id {
            if group_id > max_duplicate_group_id {
                max_duplicate_group_id = group_id;
            }
            if !file.is_original {
                duplicate_files_count += 1;
                duplicate_wasted_size = duplicate_wasted_size.saturating_add(file.size);
            }
        }
    }

    let mut category_stats = Vec::with_capacity(all_categories.len());
    let mut largest_category: Option<FileCategory> = None;
    let mut max_cat_size: u64 = 0;

    for (idx, &cat) in all_categories.iter().enumerate() {
        let size = category_sizes[idx];
        let count = category_counts[idx];

        category_stats.push(CategoryStats {
            category: cat,
            count,
            total_size: size,
        });

        if size > max_cat_size {
            max_cat_size = size;
            largest_category = Some(cat);
        }
    }

    ScanSummary {
        total_files: files.len(),
        total_size,
        category_stats,
        largest_category,
        old_files_count,
        old_files_total_size,
        duplicate_groups_count: max_duplicate_group_id,
        duplicate_files_count,
        duplicate_wasted_size,
    }
}
