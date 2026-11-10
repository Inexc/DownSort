use std::borrow::Cow;
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use gpui::*;

use core::category::FileCategory;
use core::duplicates::find_and_mark_duplicates;
use core::model::{FileItem, ScanSummary};
use core::scanner::{default_download_dir, format_bytes, open_file_in_system, scan_directory, ScanOptions};
use core::stats::compute_summary;
use core::trash_ops::{restore_from_trash, send_multiple_to_trash, send_to_trash};

// -----------------------------------------------------------------------------
// HIGH-PERFORMANCE DISCRETE GPU EXPORTS (Forces NVIDIA/AMD 180Hz Discrete GPU)
// -----------------------------------------------------------------------------
#[no_mangle]
pub static NvOptimusEnablement: u32 = 0x00000001;
#[no_mangle]
pub static AmdPowerXpressRequestHighPerformance: i32 = 1;

actions!(downsort, [ToggleFullscreen]);

// -----------------------------------------------------------------------------
// EXACT COLOR PALETTE EXTRACTED 1:1 FROM design/ref.png
// -----------------------------------------------------------------------------
const BG_WINDOW: u32 = 0x161412;           // Warm dark background #161412
const BG_SIDEBAR: u32 = 0x161412;          // Sidebar background #161412
const BG_SIDEBAR_ACTIVE: u32 = 0x211f1b;   // Active category pod #211F1B
const BG_ROW_HOVER: u32 = 0x1c1a17;        // Hovered file row
const BG_DUP_ROW: u32 = 0x1f1a19;          // Duplicate group dark brown-maroon #1F1A19
const BG_MODAL: u32 = 0x1c1a17;            // Modal background

const BORDER_PRIMARY: u32 = 0x312d27;      // Divider & ghost borders #312D27
const BORDER_SUBTLE: u32 = 0x262320;       // Very subtle separator #262320
const BORDER_CHECKBOX: u32 = 0x353534;     // Checkbox borders #353534

const TEXT_PRIMARY: u32 = 0xf3efe9;        // Off-white primary text #F3EFE9
const TEXT_SECONDARY: u32 = 0xa39b8e;      // Warm taupe secondary text #A39B8E
const TEXT_MUTED: u32 = 0x716a5f;          // Muted labels, paths, dates #716A5F
const TEXT_DUP_HEADER: u32 = 0xff9b93;     // Duplicate group header text #FF9B93

const COLOR_RED_PRIMARY: u32 = 0xe5484d;   // Soft red primary action button #E5484D
const COLOR_RED_HOVER: u32 = 0xd13b40;
const COLOR_WARN_AMBER: u32 = 0xf0a21c;    // Old files 90+ days amber #F0A21C
const BG_WARN_AMBER: u32 = 0x302512;       // Translucent amber badge background #302512
const BORDER_WARN_AMBER: u32 = 0x4a3818;   // Amber badge border #4A3818

const COLOR_TRANSPARENT: u32 = 0x00000000; // Transparent Rgba

// -----------------------------------------------------------------------------
// EMBEDDED ICONS FROM glincker/thesvg
// -----------------------------------------------------------------------------
struct AppAssets;

impl AssetSource for AppAssets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        let bytes: Option<&'static [u8]> = match path {
            "icons/word.svg" => Some(include_bytes!("../assets/icons/word.svg")),
            "icons/excel.svg" => Some(include_bytes!("../assets/icons/excel.svg")),
            "icons/pdf.svg" => Some(include_bytes!("../assets/icons/pdf.svg")),
            "icons/photo.svg" => Some(include_bytes!("../assets/icons/photo.svg")),
            "icons/zip.svg" => Some(include_bytes!("../assets/icons/zip.svg")),
            "icons/exe.svg" => Some(include_bytes!("../assets/icons/exe.svg")),
            "icons/appimage.svg" => Some(include_bytes!("../assets/icons/appimage.svg")),
            "icons/deb.svg" => Some(include_bytes!("../assets/icons/deb.svg")),
            "icons/apple.svg" => Some(include_bytes!("../assets/icons/apple.svg")),
            "icons/android.svg" => Some(include_bytes!("../assets/icons/android.svg")),
            "icons/video.svg" => Some(include_bytes!("../assets/icons/video.svg")),
            "icons/audio.svg" => Some(include_bytes!("../assets/icons/audio.svg")),
            "icons/text.svg" => Some(include_bytes!("../assets/icons/text.svg")),
            "icons/markdown.svg" => Some(include_bytes!("../assets/icons/markdown.svg")),
            "icons/disk.svg" => Some(include_bytes!("../assets/icons/disk.svg")),
            "icons/code.svg" => Some(include_bytes!("../assets/icons/code.svg")),
            "icons/file.svg" => Some(include_bytes!("../assets/icons/file.svg")),
            "icons/duplicate.svg" => Some(include_bytes!("../assets/icons/duplicate.svg")),
            "icons/logo.svg" => Some(include_bytes!("../assets/icons/logo.svg")),
            _ => None,
        };
        Ok(bytes.map(Cow::Borrowed))
    }

    fn list(&self, _path: &str) -> Result<Vec<SharedString>> {
        Ok(vec![
            "icons/word.svg".into(),
            "icons/excel.svg".into(),
            "icons/pdf.svg".into(),
            "icons/photo.svg".into(),
            "icons/zip.svg".into(),
            "icons/exe.svg".into(),
            "icons/appimage.svg".into(),
            "icons/deb.svg".into(),
            "icons/apple.svg".into(),
            "icons/android.svg".into(),
            "icons/video.svg".into(),
            "icons/audio.svg".into(),
            "icons/text.svg".into(),
            "icons/markdown.svg".into(),
            "icons/disk.svg".into(),
            "icons/code.svg".into(),
            "icons/file.svg".into(),
            "icons/duplicate.svg".into(),
            "icons/logo.svg".into(),
        ])
    }
}

pub fn get_icon_path_for_file(category: FileCategory, extension: Option<&str>) -> &'static str {
    if let Some(ext) = extension {
        let ext_lower = ext.trim_start_matches('.').to_lowercase();
        match ext_lower.as_str() {
            "doc" | "docx" | "odt" | "rtf" => return "icons/word.svg",
            "xls" | "xlsx" | "csv" | "ods" => return "icons/excel.svg",
            "pdf" => return "icons/pdf.svg",
            "jpg" | "jpeg" | "png" | "webp" | "heic" | "gif" | "bmp" | "svg" => return "icons/photo.svg",
            "zip" | "rar" | "7z" | "tar" | "gz" | "bz2" | "xz" => return "icons/zip.svg",
            "exe" | "msi" => return "icons/exe.svg",
            "deb" | "rpm" => return "icons/deb.svg",
            "appimage" => return "icons/appimage.svg",
            "dmg" | "pkg" => return "icons/apple.svg",
            "apk" => return "icons/android.svg",
            "mp4" | "mkv" | "avi" | "mov" | "webm" => return "icons/video.svg",
            "mp3" | "wav" | "flac" | "ogg" | "m4a" => return "icons/audio.svg",
            "txt" | "log" | "ini" | "cfg" | "conf" => return "icons/text.svg",
            "md" | "markdown" => return "icons/markdown.svg",
            "iso" | "img" | "vmdk" => return "icons/disk.svg",
            "rs" | "js" | "ts" | "py" | "c" | "cpp" | "h" | "html" | "css" | "json" | "toml" | "yaml" | "xml" => return "icons/code.svg",
            _ => {}
        }
    }

    match category {
        FileCategory::Documents => "icons/word.svg",
        FileCategory::Pdf => "icons/pdf.svg",
        FileCategory::Spreadsheets => "icons/excel.svg",
        FileCategory::Photos => "icons/photo.svg",
        FileCategory::Archives => "icons/zip.svg",
        FileCategory::Installers => "icons/exe.svg",
        FileCategory::Media => "icons/video.svg",
        FileCategory::Other => "icons/file.svg",
    }
}

pub fn get_icon_path_for_category(category: Option<FileCategory>) -> &'static str {
    match category {
        None => "icons/file.svg",
        Some(FileCategory::Documents) => "icons/word.svg",
        Some(FileCategory::Pdf) => "icons/pdf.svg",
        Some(FileCategory::Spreadsheets) => "icons/excel.svg",
        Some(FileCategory::Photos) => "icons/photo.svg",
        Some(FileCategory::Archives) => "icons/zip.svg",
        Some(FileCategory::Installers) => "icons/exe.svg",
        Some(FileCategory::Media) => "icons/video.svg",
        Some(FileCategory::Other) => "icons/file.svg",
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortField {
    Size,
    Date,
    Name,
}

#[derive(Clone)]
pub struct UndoState {
    pub id: u64,
    pub files: Vec<FileItem>,
    pub count: usize,
    pub total_size: u64,
    pub expires_at: Instant,
}

pub struct DownSortApp {
    scan_path: PathBuf,
    files: Vec<FileItem>,
    summary: ScanSummary,
    selected_category: Option<FileCategory>,
    filter_duplicates_only: bool,
    filter_old_only: bool,
    sort_field: SortField,
    sort_descending: bool,
    is_scanning: bool,
    is_hashing: bool,
    is_deleting: bool,
    status_message: Option<String>,
    show_confirm_dialog: bool,
    // Performance cache
    filtered_files: Vec<FileItem>,
    display_limit: usize,
    // Undo
    undo_state: Option<UndoState>,
    undo_id: u64,
    // Animation versions
    filter_version: u64,
    donut_anim_version: u64,
}

impl DownSortApp {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        let scan_path = default_download_dir();
        let mut app = Self {
            scan_path: scan_path.clone(),
            files: Vec::new(),
            summary: ScanSummary::default(),
            selected_category: None,
            filter_duplicates_only: false,
            filter_old_only: false,
            sort_field: SortField::Size,
            sort_descending: true,
            is_scanning: false,
            is_hashing: false,
            is_deleting: false,
            status_message: Some("Загрузка папки...".to_string()),
            show_confirm_dialog: false,
            filtered_files: Vec::new(),
            display_limit: 100,
            undo_state: None,
            undo_id: 0,
            filter_version: 0,
            donut_anim_version: 0,
        };
        app.run_scan_sync();
        app
    }

    /// Update cached filtered and sorted list (avoids expensive sorting on every render frame)
    fn update_filtered_cache(&mut self) {
        self.filter_version += 1;
        let mut items: Vec<FileItem> = self
            .files
            .iter()
            .filter(|f| {
                if self.filter_duplicates_only {
                    f.duplicate_group_id.is_some()
                } else if self.filter_old_only {
                    f.is_old
                } else if let Some(cat) = self.selected_category {
                    f.category == cat
                } else {
                    true
                }
            })
            .cloned()
            .collect();

        let sort_field = self.sort_field;
        let sort_descending = self.sort_descending;

        items.sort_by(|a, b| {
            // Keep duplicate groups clustered together
            if a.duplicate_group_id.is_some() && b.duplicate_group_id.is_some() {
                if a.duplicate_group_id != b.duplicate_group_id {
                    return a.duplicate_group_id.cmp(&b.duplicate_group_id);
                }
                if a.is_original != b.is_original {
                    return b.is_original.cmp(&a.is_original);
                }
            }

            let ordering = match sort_field {
                SortField::Size => a.size.cmp(&b.size),
                SortField::Date => a.last_accessed.cmp(&b.last_accessed),
                SortField::Name => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            };

            if sort_descending {
                ordering.reverse()
            } else {
                ordering
            }
        });

        self.filtered_files = items;
    }

    /// Perform initial synchronous scan of the default Downloads folder
    fn run_scan_sync(&mut self) {
        let options = ScanOptions {
            root_path: self.scan_path.clone(),
            max_depth: 2,
            old_threshold_days: 90,
        };

        match scan_directory(&options) {
            Ok(scanned_files) => {
                self.summary = compute_summary(&scanned_files);
                self.files = scanned_files;
                self.update_filtered_cache();
                self.status_message = Some(format!(
                    "Найдено файлов: {} ({})",
                    self.summary.total_files,
                    format_bytes(self.summary.total_size)
                ));
            }
            Err(e) => {
                self.status_message = Some(format!("Ошибка сканирования: {}", e));
            }
        }
    }

    /// Open a file in the system default program
    pub fn open_file(&self, id: usize) {
        if let Some(file) = self.files.iter().find(|f| f.id == id) {
            let _ = open_file_in_system(&file.path);
        }
    }

    /// Trigger re-scan in a background thread so UI remains responsive
    pub fn trigger_rescan(&mut self, cx: &mut Context<Self>) {
        if self.is_scanning || self.is_hashing || self.is_deleting {
            return;
        }

        self.is_scanning = true;
        self.status_message = Some("Сканирование папки...".to_string());
        cx.notify();

        let path = self.scan_path.clone();
        let (tx, rx) = mpsc::channel();

        thread::spawn(move || {
            let options = ScanOptions {
                root_path: path,
                max_depth: 2,
                old_threshold_days: 90,
            };
            let res = scan_directory(&options);
            let _ = tx.send(res);
        });

        cx.spawn(async move |this, cx| {
            let res = rx.recv();
            let _ = this.update(cx, |app, cx| {
                app.is_scanning = false;
                match res {
                    Ok(Ok(scanned_files)) => {
                        app.summary = compute_summary(&scanned_files);
                        app.files = scanned_files;
                        app.update_filtered_cache();
                        app.status_message = Some(format!(
                            "Сканирование завершено: {} файлов ({})",
                            app.summary.total_files,
                            format_bytes(app.summary.total_size)
                        ));
                    }
                    Ok(Err(e)) => {
                        app.status_message = Some(format!("Ошибка сканирования: {}", e));
                    }
                    Err(_) => {
                        app.status_message = Some("Ошибка потока сканирования".to_string());
                    }
                }
                cx.notify();
            }).ok();
        })
        .detach();
    }

    /// Trigger duplicate finder in a background thread
    pub fn trigger_find_duplicates(&mut self, cx: &mut Context<Self>) {
        if self.is_scanning || self.is_hashing || self.is_deleting || self.files.is_empty() {
            return;
        }

        self.is_hashing = true;
        self.status_message = Some("Поиск дубликатов (хеширование SHA-256)...".to_string());
        cx.notify();

        let mut files_to_hash = self.files.clone();
        let (tx, rx) = mpsc::channel();

        thread::spawn(move || {
            let groups = find_and_mark_duplicates(&mut files_to_hash);
            let summary = compute_summary(&files_to_hash);
            let _ = tx.send((files_to_hash, groups, summary));
        });

        cx.spawn(async move |this, cx| {
            let res = rx.recv();
            let _ = this.update(cx, |app, cx| {
                app.is_hashing = false;
                if let Ok((updated_files, groups, summary)) = res {
                    app.files = updated_files;
                    app.summary = summary;
                    let count = groups.len();
                    if count > 0 {
                        app.status_message = Some(format!(
                            "Найдено групп дубликатов: {} (лишних файлов: {}, можно освободить {})",
                            count,
                            app.summary.duplicate_files_count,
                            format_bytes(app.summary.duplicate_wasted_size)
                        ));
                        app.filter_duplicates_only = true;
                    } else {
                        app.status_message = Some("Дубликатов не обнаружено".to_string());
                    }
                    app.update_filtered_cache();
                }
                cx.notify();
            }).ok();
        })
        .detach();
    }

    /// Toggle file checkbox
    pub fn toggle_file_selection(&mut self, id: usize, cx: &mut Context<Self>) {
        if let Some(item) = self.files.iter_mut().find(|f| f.id == id) {
            item.selected_for_deletion = !item.selected_for_deletion;
            let new_val = item.selected_for_deletion;
            if let Some(cached) = self.filtered_files.iter_mut().find(|f| f.id == id) {
                cached.selected_for_deletion = new_val;
            }
            cx.notify();
        }
    }

    /// Select or deselect all currently filtered files
    pub fn select_all_filtered(&mut self, select: bool, cx: &mut Context<Self>) {
        let matching_ids: Vec<usize> = self.filtered_files.iter().map(|f| f.id).collect();
        for item in self.files.iter_mut() {
            if matching_ids.contains(&item.id) {
                item.selected_for_deletion = select;
            }
        }
        for cached in self.filtered_files.iter_mut() {
            cached.selected_for_deletion = select;
        }
        cx.notify();
    }

    /// Single file quick delete to Trash
    pub fn delete_single_file(&mut self, id: usize, cx: &mut Context<Self>) {
        if let Some(pos) = self.files.iter().position(|f| f.id == id) {
            let item = self.files[pos].clone();
            let path = item.path.clone();
            let name = item.name.clone();
            match send_to_trash(&path) {
                Ok(()) => {
                    self.files.remove(pos);
                    self.summary = compute_summary(&self.files);
                    self.donut_anim_version += 1;
                    self.update_filtered_cache();
                    self.status_message = Some(format!("Файл «{}» отправлен в Корзину", name));
                    self.register_undo(vec![item], cx);
                }
                Err(e) => {
                    self.status_message = Some(format!("Ошибка удаления «{}»: {}", name, e));
                }
            }
            cx.notify();
        }
    }

    /// Confirm and execute mass deletion of selected files in a background thread
    pub fn execute_mass_trash(&mut self, cx: &mut Context<Self>) {
        self.show_confirm_dialog = false;

        let selected_paths: Vec<PathBuf> = self
            .files
            .iter()
            .filter(|f| f.selected_for_deletion)
            .map(|f| f.path.clone())
            .collect();

        if selected_paths.is_empty() {
            return;
        }

        self.is_deleting = true;
        self.status_message = Some(format!("Перемещение {} файлов в Корзину...", selected_paths.len()));
        cx.notify();

        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            let res = send_multiple_to_trash(&selected_paths);
            let _ = tx.send(res);
        });

        cx.spawn(async move |this, cx| {
            let res = rx.recv();
            let _ = this.update(cx, |app, cx| {
                app.is_deleting = false;
                if let Ok(res) = res {
                    let deleted_count = res.deleted.len();
                    let failed_count = res.failed.len();

                    let deleted_items: Vec<FileItem> = app
                        .files
                        .iter()
                        .filter(|f| res.deleted.contains(&f.path))
                        .cloned()
                        .collect();

                    app.files.retain(|f| !res.deleted.contains(&f.path));
                    app.summary = compute_summary(&app.files);
                    app.donut_anim_version += 1;
                    app.update_filtered_cache();

                    if !deleted_items.is_empty() {
                        app.register_undo(deleted_items, cx);
                    }

                    if failed_count == 0 {
                        app.status_message = Some(format!(
                            "Успешно перемещено в Корзину {} файлов",
                            deleted_count
                        ));
                    } else {
                        app.status_message = Some(format!(
                            "В Корзину перемещено {} файлов (ошибок: {})",
                            deleted_count, failed_count
                        ));
                    }
                }
                cx.notify();
            }).ok();
        })
        .detach();
    }

    /// Register deleted files for temporary Undo toast
    pub fn register_undo(&mut self, items: Vec<FileItem>, cx: &mut Context<Self>) {
        if items.is_empty() {
            return;
        }
        let count = items.len();
        let total_size: u64 = items.iter().map(|f| f.size).sum();
        self.undo_id += 1;
        let current_id = self.undo_id;
        self.undo_state = Some(UndoState {
            id: current_id,
            files: items,
            count,
            total_size,
            expires_at: Instant::now() + Duration::from_secs(7),
        });

        cx.spawn(async move |this, cx| {
            cx.background_executor().timer(Duration::from_secs(7)).await;
            let _ = this.update(cx, |this, cx| {
                if let Some(state) = &this.undo_state {
                    if state.id == current_id {
                        this.undo_state = None;
                        cx.notify();
                    }
                }
            });
        }).detach();
    }

    /// Restore deleted files from the system trash back to original paths
    pub fn trigger_undo(&mut self, cx: &mut Context<Self>) {
        let Some(undo) = self.undo_state.take() else {
            return;
        };

        self.status_message = Some("Восстановление файлов из Корзины...".to_string());
        cx.notify();

        let paths: Vec<PathBuf> = undo.files.iter().map(|f| f.path.clone()).collect();
        let (tx, rx) = mpsc::channel();

        thread::spawn(move || {
            let res = restore_from_trash(&paths);
            let _ = tx.send(res);
        });

        let restored_files = undo.files;
        cx.spawn(async move |this, cx| {
            let res = rx.recv();
            let _ = this.update(cx, |app, cx| {
                match res {
                    Ok(Ok(restore_result)) => {
                        let restored_count = restore_result.restored.len();
                        let failed_count = restore_result.failed.len();

                        for file in restored_files {
                            if restore_result.restored.contains(&file.path) {
                                if !app.files.iter().any(|f| f.path == file.path) {
                                    app.files.push(file);
                                }
                            }
                        }

                        app.summary = compute_summary(&app.files);
                        app.donut_anim_version += 1;
                        app.update_filtered_cache();

                        if failed_count == 0 {
                            app.status_message = Some(format!(
                                "Успешно восстановлено {} файлов из Корзины",
                                restored_count
                            ));
                        } else {
                            app.status_message = Some(format!(
                                "Восстановлено {} файлов (не удалось: {})",
                                restored_count, failed_count
                            ));
                        }
                    }
                    Ok(Err(e)) => {
                        app.status_message = Some(format!("Ошибка восстановления: {}", e));
                    }
                    Err(_) => {
                        app.status_message = Some("Ошибка потока восстановления".to_string());
                    }
                }
                cx.notify();
            }).ok();
        }).detach();
    }

    /// Count and total size of selected files
    fn get_selected_stats(&self) -> (usize, u64) {
        let mut count = 0;
        let mut size = 0;
        for f in &self.files {
            if f.selected_for_deletion {
                count += 1;
                size += f.size;
            }
        }
        (count, size)
    }

    /// Render Donut Chart with segmented category colors and centered volume
    fn render_donut(&self) -> impl IntoElement {
        let (num_str, unit_str) = split_bytes(self.summary.total_size);
        let stats: Vec<(FileCategory, u64)> = self
            .summary
            .category_stats
            .iter()
            .filter(|s| s.total_size > 0)
            .map(|s| (s.category, s.total_size))
            .collect();
        let total_size = self.summary.total_size;

        div()
            .relative()
            .w(px(108.))
            .h(px(108.))
            .flex()
            .items_center()
            .justify_center()
            .child(
                canvas(
                    move |_bounds, _window, _cx| (stats, total_size),
                    move |bounds, (stats, total_size), window, _cx| {
                        let center = bounds.center();
                        let radius_px = bounds.size.width.min(bounds.size.height) / 2.0;
                        let radius = f32::from(&radius_px);

                        if total_size == 0 || stats.is_empty() {
                            let mut path = Path::new(center);
                            let steps = 40;
                            for i in 0..=steps {
                                let angle = (i as f32 / steps as f32) * std::f32::consts::TAU;
                                let pt = point(
                                    center.x + px(radius * angle.cos()),
                                    center.y + px(radius * angle.sin()),
                                );
                                path.line_to(pt);
                            }
                            path.line_to(center);
                            window.paint_path(path, rgb(BORDER_PRIMARY));
                            return;
                        }

                        let mut current_angle = -std::f32::consts::FRAC_PI_2;
                        let gap = 0.05;

                        for (cat, size) in stats {
                            let fraction = size as f32 / total_size as f32;
                            let sweep = fraction * std::f32::consts::TAU;
                            if sweep < 0.04 {
                                continue;
                            }

                            let start_angle = current_angle + gap / 2.0;
                            let end_angle = current_angle + sweep - gap / 2.0;
                            current_angle += sweep;

                            if end_angle <= start_angle {
                                continue;
                            }

                            let (r, g, b) = cat.rgb();
                            let seg_color = rgb(((r as u32) << 16) | ((g as u32) << 8) | (b as u32));

                            let mut path = Path::new(center);
                            let steps = ((end_angle - start_angle) * 16.0).max(6.0) as usize;
                            for i in 0..=steps {
                                let t = i as f32 / steps as f32;
                                let angle = start_angle + t * (end_angle - start_angle);
                                let pt = point(
                                    center.x + px(radius * angle.cos()),
                                    center.y + px(radius * angle.sin()),
                                );
                                path.line_to(pt);
                            }
                            path.line_to(center);
                            window.paint_path(path, seg_color);
                        }
                    },
                )
                .size_full()
            )
            .child(
                div()
                    .absolute()
                    .w(px(74.))
                    .h(px(74.))
                    .rounded_full()
                    .bg(rgb(BG_WINDOW))
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .child(
                        div()
                            .text_2xl()
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(TEXT_PRIMARY))
                            .child(num_str)
                    )
                    .child(
                        div()
                            .text_xs()
                            .font_weight(FontWeight::NORMAL)
                            .text_color(rgb(TEXT_SECONDARY))
                            .child(unit_str)
                    )
            )
            .with_animation(
                SharedString::from(format!("donut-anim-{}", self.donut_anim_version)),
                Animation::new(Duration::from_millis(300)).with_easing(ease_in_out),
                |d, delta| d.opacity(delta),
            )
    }
}

/// Helper to split byte formatting into number and unit (e.g. ("4.2", "ГБ"))
fn split_bytes(bytes: u64) -> (String, String) {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;
    const TB: f64 = GB * 1024.0;

    let b = bytes as f64;
    if b >= TB {
        (format!("{:.1}", b / TB), "ТБ".to_string())
    } else if b >= GB {
        (format!("{:.1}", b / GB), "ГБ".to_string())
    } else if b >= MB {
        (format!("{:.0}", b / MB), "МБ".to_string())
    } else if b >= KB {
        (format!("{:.0}", b / KB), "КБ".to_string())
    } else {
        (format!("{}", bytes), "Б".to_string())
    }
}

/// Russian localized date formatting (matching reference "14 сен 2026 · не открывался")
fn format_date_label(time: Option<SystemTime>, is_old: bool) -> String {
    let Some(t) = time else {
        return "—".to_string();
    };

    let duration = match t.duration_since(UNIX_EPOCH) {
        Ok(d) => d,
        Err(_) => return "—".to_string(),
    };

    let total_secs = duration.as_secs();
    let days_since_epoch = total_secs / 86400;

    let z = (days_since_epoch as i64) + 719468;
    let era = z / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = mp + (if mp < 10 { 3 } else { -9 });
    let year = y + (if m <= 2 { 1 } else { 0 });

    let month_str = match m {
        1 => "янв",
        2 => "фев",
        3 => "мар",
        4 => "апр",
        5 => "мая",
        6 => "июн",
        7 => "июл",
        8 => "авг",
        9 => "сен",
        10 => "окт",
        11 => "ноя",
        12 => "дек",
        _ => "",
    };

    if is_old {
        format!("{} {} {} · не открывался", d, month_str, year)
    } else {
        format!("{} {} {}", d, month_str, year)
    }
}

impl Render for DownSortApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let (selected_count, selected_size) = self.get_selected_stats();
        let is_modal_open = self.show_confirm_dialog;

        let summary_desc = match (self.summary.largest_category, self.summary.total_size) {
            (Some(cat), s) if s > 0 => {
                let cat_size = self
                    .summary
                    .category_stats
                    .iter()
                    .find(|c| c.category == cat)
                    .map(|c| c.total_size)
                    .unwrap_or(0);
                format!(
                    "Больше всего места занимает {} ({}).",
                    cat.display_name(),
                    format_bytes(cat_size)
                )
            }
            _ => "Папка пуста или файлы не найдены.".to_string(),
        };

        let is_fullscreen = window.is_fullscreen();
        let is_maximized = window.is_maximized();

        div()
            .on_action(cx.listener(|_this, _: &ToggleFullscreen, window, _cx| {
                window.toggle_fullscreen();
            }))
            .on_key_down(cx.listener(|_this, event: &KeyDownEvent, window, _cx| {
                if event.keystroke.key == "f11" || (event.keystroke.modifiers.alt && event.keystroke.key == "enter") {
                    window.toggle_fullscreen();
                }
            }))
            .flex()
            .flex_row()
            .size_full()
            .bg(rgb(BG_WINDOW))
            .text_color(rgb(TEXT_PRIMARY))
            // -------------------------------------------------------------
            // SIDEBAR (Left)
            // -------------------------------------------------------------
            .child(
                div()
                    .flex()
                    .flex_col()
                    .w(px(230.))
                    .h_full()
                    .bg(rgb(BG_SIDEBAR))
                    .border_r_1()
                    .border_color(rgb(BORDER_PRIMARY))
                    .p(px(14.))
                    .gap(px(10.))
                    // Header with Brand Logo & Title (Draggable window control area)
                    .child(
                        div()
                            .id("sidebar-drag-header")
                            .flex()
                            .flex_row()
                            .items_center()
                            .w_full()
                            .gap(px(8.))
                            .px(px(6.))
                            .pt(px(2.))
                            .pb(px(8.))
                            .window_control_area(WindowControlArea::Drag)
                            .child(
                                img("icons/logo.svg")
                                    .w(px(20.))
                                    .h(px(20.))
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(rgb(TEXT_PRIMARY))
                                    .child("DownSort")
                            )
                    )
                    // Directory Label
                    .child(
                        div()
                            .px(px(8.))
                            .py(px(4.))
                            .text_xs()
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(TEXT_MUTED))
                            .child("~/DOWNLOADS")
                    )
                    // Category Filters List
                    .child(
                        div()
                            .id("category-sidebar-scroll")
                            .flex()
                            .flex_col()
                            .gap(px(2.))
                            .overflow_y_scroll()
                            // "Все файлы" button
                            .child({
                                let is_all_active = !self.filter_duplicates_only && !self.filter_old_only && self.selected_category.is_none();
                                let btn = div()
                                    .id("btn-all-files")
                                    .flex()
                                    .flex_row()
                                    .justify_between()
                                    .items_center()
                                    .px(px(10.))
                                    .py(px(7.))
                                    .rounded(px(7.))
                                    .bg(if is_all_active {
                                        rgb(BG_SIDEBAR_ACTIVE)
                                    } else {
                                        rgba(COLOR_TRANSPARENT)
                                    })
                                    .border_1()
                                    .border_color(if is_all_active {
                                        rgb(BORDER_PRIMARY)
                                    } else {
                                        rgba(COLOR_TRANSPARENT)
                                    })
                                    .hover(|s| s.bg(rgb(BG_ROW_HOVER)))
                                    .cursor_pointer()
                                    .on_click(cx.listener(|this, _event, _window, cx| {
                                        this.filter_duplicates_only = false;
                                        this.filter_old_only = false;
                                        this.selected_category = None;
                                        this.update_filtered_cache();
                                        cx.notify();
                                    }))
                                    .child(
                                        div()
                                            .flex()
                                            .flex_row()
                                            .items_center()
                                            .gap(px(10.))
                                            .child(
                                                img("icons/file.svg")
                                                    .size(px(16.))
                                            )
                                            .child(
                                                div()
                                                    .text_sm()
                                                    .font_weight(if is_all_active {
                                                        FontWeight::MEDIUM
                                                    } else {
                                                        FontWeight::NORMAL
                                                    })
                                                    .text_color(if is_all_active {
                                                        rgb(TEXT_PRIMARY)
                                                    } else {
                                                        rgb(TEXT_SECONDARY)
                                                    })
                                                    .child("Все файлы")
                                            )
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(rgb(TEXT_MUTED))
                                            .child(format!("{}", self.summary.total_files))
                                    );

                                if is_all_active {
                                    btn.with_animation(
                                        "pod-anim-all",
                                        Animation::new(Duration::from_millis(150)).with_easing(ease_in_out),
                                        |b, delta| b.opacity(delta),
                                    ).into_any_element()
                                } else {
                                    btn.into_any_element()
                                }
                            })
                            // Standard Categories
                            .children(FileCategory::all().iter().enumerate().map(|(idx, &cat)| {
                                let is_active = !self.filter_duplicates_only && !self.filter_old_only && self.selected_category == Some(cat);
                                let stat = self
                                    .summary
                                    .category_stats
                                    .iter()
                                    .find(|s| s.category == cat);
                                let count = stat.map(|s| s.count).unwrap_or(0);

                                let btn = div()
                                    .id(SharedString::from(format!("cat-btn-{}", idx)))
                                    .flex()
                                    .flex_row()
                                    .justify_between()
                                    .items_center()
                                    .px(px(10.))
                                    .py(px(7.))
                                    .rounded(px(7.))
                                    .bg(if is_active {
                                        rgb(BG_SIDEBAR_ACTIVE)
                                    } else {
                                        rgba(COLOR_TRANSPARENT)
                                    })
                                    .border_1()
                                    .border_color(if is_active {
                                        rgb(BORDER_PRIMARY)
                                    } else {
                                        rgba(COLOR_TRANSPARENT)
                                    })
                                    .hover(|s| s.bg(rgb(BG_ROW_HOVER)))
                                    .cursor_pointer()
                                    .on_click(cx.listener(move |this, _event, _window, cx| {
                                        this.filter_duplicates_only = false;
                                        this.filter_old_only = false;
                                        this.selected_category = Some(cat);
                                        this.update_filtered_cache();
                                        cx.notify();
                                    }))
                                    .child(
                                        div()
                                            .flex()
                                            .flex_row()
                                            .items_center()
                                            .gap(px(10.))
                                            .child(
                                                img(get_icon_path_for_category(Some(cat)))
                                                    .size(px(16.))
                                            )
                                            .child(
                                                div()
                                                    .text_sm()
                                                    .font_weight(if is_active { FontWeight::MEDIUM } else { FontWeight::NORMAL })
                                                    .text_color(if is_active { rgb(TEXT_PRIMARY) } else { rgb(TEXT_SECONDARY) })
                                                    .child(cat.display_name())
                                            )
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(rgb(TEXT_MUTED))
                                            .child(format!("{}", count))
                                    );

                                if is_active {
                                    btn.with_animation(
                                        SharedString::from(format!("pod-anim-{}", idx)),
                                        Animation::new(Duration::from_millis(150)).with_easing(ease_in_out),
                                        |b, delta| b.opacity(delta),
                                    ).into_any_element()
                                } else {
                                    btn.into_any_element()
                                }
                            }))
                            .child(div().my(px(6.)).h(px(1.)).bg(rgb(BORDER_SUBTLE)))
                            // "Дубликаты" virtual category
                            .child({
                                let is_dup_active = self.filter_duplicates_only;
                                let btn = div()
                                    .id("btn-duplicates")
                                    .flex()
                                    .flex_row()
                                    .justify_between()
                                    .items_center()
                                    .px(px(10.))
                                    .py(px(7.))
                                    .rounded(px(7.))
                                    .bg(if is_dup_active {
                                        rgb(0x281615)
                                    } else {
                                        rgba(COLOR_TRANSPARENT)
                                    })
                                    .border_1()
                                    .border_color(if is_dup_active {
                                        rgb(0x4a2220)
                                    } else {
                                        rgba(COLOR_TRANSPARENT)
                                    })
                                    .hover(|s| s.bg(rgb(0x201514)))
                                    .cursor_pointer()
                                    .on_click(cx.listener(|this, _event, _window, cx| {
                                        this.filter_duplicates_only = true;
                                        this.filter_old_only = false;
                                        this.selected_category = None;
                                        this.update_filtered_cache();
                                        cx.notify();
                                    }))
                                    .child(
                                        div()
                                            .flex()
                                            .flex_row()
                                            .items_center()
                                            .gap(px(10.))
                                            .child(
                                                img("icons/duplicate.svg")
                                                    .size(px(16.))
                                            )
                                            .child(
                                                div()
                                                    .text_sm()
                                                    .font_weight(FontWeight::MEDIUM)
                                                    .text_color(if is_dup_active { rgb(TEXT_PRIMARY) } else { rgb(TEXT_SECONDARY) })
                                                    .child("Дубликаты")
                                            )
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(if self.summary.duplicate_files_count > 0 {
                                                rgb(TEXT_DUP_HEADER)
                                            } else {
                                                rgb(TEXT_MUTED)
                                            })
                                            .child(format!("{}", self.summary.duplicate_files_count))
                                    );

                                if is_dup_active {
                                    btn.with_animation(
                                        "pod-anim-dup",
                                        Animation::new(Duration::from_millis(150)).with_easing(ease_in_out),
                                        |b, delta| b.opacity(delta),
                                    ).into_any_element()
                                } else {
                                    btn.into_any_element()
                                }
                            })
                    )
                    // Bottom Sidebar Rescan Button (Ghost Style with thin border)
                    .child(
                        div()
                            .mt_auto()
                            .pt(px(10.))
                            .child(
                                div()
                                    .id("btn-rescan")
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .gap(px(6.))
                                    .w_full()
                                    .py(px(8.))
                                    .rounded(px(8.))
                                    .border_1()
                                    .border_color(rgb(BORDER_PRIMARY))
                                    .bg(rgba(COLOR_TRANSPARENT))
                                    .hover(|s| s.bg(rgb(BG_ROW_HOVER)))
                                    .cursor_pointer()
                                    .on_click(cx.listener(|this, _event, _window, cx| {
                                        this.trigger_rescan(cx);
                                    }))
                                    .child(
                                        div()
                                            .text_xs()
                                            .font_weight(FontWeight::MEDIUM)
                                            .text_color(rgb(TEXT_SECONDARY))
                                            .child(if self.is_scanning { "⟳ Сканирование..." } else { "⟳ Пересканировать" })
                                    )
                            )
                    )
            )
            // -------------------------------------------------------------
            // MAIN CONTENT (Right)
            // -------------------------------------------------------------
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .min_w_0()
                    .h_full()
                    .px(px(24.))
                    .pb(px(24.))
                    .pt(px(10.))
                    .gap(px(12.))
                    // TOP WINDOW BAR (Window controls & Drag region)
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .justify_between()
                            .w_full()
                            .h(px(28.))
                            // Draggable region taking all available space
                            .child(
                                div()
                                    .id("main-drag-region")
                                    .flex_1()
                                    .h_full()
                                    .window_control_area(WindowControlArea::Drag)
                            )
                            // Window Control Buttons
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .gap(px(4.))
                                    // Fullscreen Toggle Button (F11)
                                    .child(
                                        div()
                                            .id("btn-win-fullscreen")
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .w(px(28.))
                                            .h(px(24.))
                                            .rounded(px(6.))
                                            .text_xs()
                                            .text_color(rgb(TEXT_MUTED))
                                            .hover(|s| s.bg(rgb(BG_ROW_HOVER)).text_color(rgb(TEXT_PRIMARY)))
                                            .cursor_pointer()
                                            .on_click(cx.listener(|_this, _event, window, _cx| {
                                                window.toggle_fullscreen();
                                            }))
                                            .child(if is_fullscreen { "⇲" } else { "⛶" })
                                    )
                                    // Minimize Button
                                    .child(
                                        div()
                                            .id("btn-win-minimize")
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .w(px(28.))
                                            .h(px(24.))
                                            .rounded(px(6.))
                                            .text_xs()
                                            .text_color(rgb(TEXT_MUTED))
                                            .hover(|s| s.bg(rgb(BG_ROW_HOVER)).text_color(rgb(TEXT_PRIMARY)))
                                            .cursor_pointer()
                                            .window_control_area(WindowControlArea::Min)
                                            .on_click(cx.listener(|_this, _event, window, _cx| {
                                                window.minimize_window();
                                            }))
                                            .child("─")
                                    )
                                    // Maximize / Restore Button
                                    .child(
                                        div()
                                            .id("btn-win-maximize")
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .w(px(28.))
                                            .h(px(24.))
                                            .rounded(px(6.))
                                            .text_xs()
                                            .text_color(rgb(TEXT_MUTED))
                                            .hover(|s| s.bg(rgb(BG_ROW_HOVER)).text_color(rgb(TEXT_PRIMARY)))
                                            .cursor_pointer()
                                            .window_control_area(WindowControlArea::Max)
                                            .on_click(cx.listener(|_this, _event, window, _cx| {
                                                window.zoom_window();
                                            }))
                                            .child(if is_maximized { "❐" } else { "□" })
                                    )
                                    // Close Button
                                    .child(
                                        div()
                                            .id("btn-win-close")
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .w(px(28.))
                                            .h(px(24.))
                                            .rounded(px(6.))
                                            .text_xs()
                                            .text_color(rgb(TEXT_MUTED))
                                            .hover(|s| s.bg(rgb(COLOR_RED_PRIMARY)).text_color(rgb(0xffffff)))
                                            .cursor_pointer()
                                            .window_control_area(WindowControlArea::Close)
                                            .on_click(cx.listener(|_this, _event, _window, cx| {
                                                cx.quit();
                                            }))
                                            .child("✕")
                                    )
                            )
                    )
                    // TOP SUMMARY SECTION WITH DONUT CHART
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .w_full()
                            .items_center()
                            .gap(px(24.))
                            .pb(px(20.))
                            .border_b_1()
                            .border_color(rgb(BORDER_PRIMARY))
                            // Donut Chart
                            .child(self.render_donut())
                            // Metrics & Warning
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .flex_1()
                                    .items_start()
                                    .gap(px(6.))
                                    .child(
                                        div()
                                            .text_xl()
                                            .font_weight(FontWeight::BOLD)
                                            .text_color(rgb(TEXT_PRIMARY))
                                            .child(format!(
                                                "{} файлов · {}",
                                                self.summary.total_files,
                                                format_bytes(self.summary.total_size)
                                            ))
                                    )
                                    .child(
                                        div()
                                            .text_sm()
                                            .text_color(rgb(TEXT_SECONDARY))
                                            .child(summary_desc)
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(rgb(TEXT_MUTED))
                                            .child("Последнее сканирование: только что.")
                                    )
                                    .children(if self.summary.old_files_count > 0 {
                                        vec![
                                            div()
                                                .id("banner-old-files")
                                                .flex()
                                                .flex_row()
                                                .items_center()
                                                .justify_between()
                                                .gap(px(14.))
                                                .px(px(14.))
                                                .py(px(8.))
                                                .mt(px(6.))
                                                .rounded(px(8.))
                                                .bg(if self.filter_old_only {
                                                    rgba(0x422a10ee)
                                                } else {
                                                    rgba(0x281d0ecc)
                                                })
                                                .border_1()
                                                .border_color(if self.filter_old_only {
                                                    rgb(COLOR_WARN_AMBER)
                                                } else {
                                                    rgba(0xf0a21c50)
                                                })
                                                .hover(|s| s.border_color(rgb(COLOR_WARN_AMBER)).bg(rgba(0x352310ee)))
                                                .cursor_pointer()
                                                .on_click(cx.listener(|this, _event, _window, cx| {
                                                    this.filter_old_only = !this.filter_old_only;
                                                    if this.filter_old_only {
                                                        this.filter_duplicates_only = false;
                                                        this.selected_category = None;
                                                    }
                                                    this.update_filtered_cache();
                                                    cx.notify();
                                                }))
                                                .child(
                                                    div()
                                                        .flex()
                                                        .flex_row()
                                                        .items_center()
                                                        .gap(px(10.))
                                                        .child(
                                                            div()
                                                                .text_sm()
                                                                .child("⚠️")
                                                        )
                                                        .child(
                                                            div()
                                                                .text_xs()
                                                                .font_weight(FontWeight::MEDIUM)
                                                                .text_color(rgb(COLOR_WARN_AMBER))
                                                                .child(format!(
                                                                    "{} файлов не открывались 90+ дней · занимают {}",
                                                                    self.summary.old_files_count,
                                                                    format_bytes(self.summary.old_files_total_size)
                                                                ))
                                                        )
                                                )
                                                .child(
                                                    div()
                                                        .id("btn-toggle-old-files")
                                                        .text_xs()
                                                        .font_weight(FontWeight::BOLD)
                                                        .text_color(rgb(TEXT_PRIMARY))
                                                        .hover(|s| s.underline().text_color(rgb(0xffffff)))
                                                        .child(if self.filter_old_only {
                                                            "Показать все ✕"
                                                        } else {
                                                            "Показать →"
                                                        })
                                                )
                                                .into_any_element()
                                        ]
                                    } else {
                                        vec![]
                                    })
                            )
                    )
                    // TOOLBAR
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .justify_between()
                            .items_center()
                            .w_full()
                            .py(px(4.))
                            // Left: Selection Counter & Select/Deselect
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .gap(px(8.))
                                    .child(
                                        div()
                                            .text_sm()
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .text_color(rgb(TEXT_PRIMARY))
                                            .child(format!("{} выбрано", selected_count))
                                    )
                                    .child(div().text_sm().text_color(rgb(TEXT_MUTED)).child("·"))
                                    .child(
                                        div()
                                            .text_sm()
                                            .text_color(rgb(TEXT_SECONDARY))
                                            .child(format_bytes(selected_size))
                                    )
                                    .child(div().w(px(10.)))
                                    .child(
                                        div()
                                            .id("btn-select-all")
                                            .text_xs()
                                            .text_color(rgb(TEXT_MUTED))
                                            .hover(|s| s.text_color(rgb(TEXT_PRIMARY)))
                                            .cursor_pointer()
                                            .on_click(cx.listener(|this, _event, _window, cx| {
                                                this.select_all_filtered(true, cx);
                                            }))
                                            .child("Выбрать все")
                                    )
                                    .child(div().text_xs().text_color(rgb(BORDER_PRIMARY)).child("|"))
                                    .child(
                                        div()
                                            .id("btn-deselect-all")
                                            .text_xs()
                                            .text_color(rgb(TEXT_MUTED))
                                            .hover(|s| s.text_color(rgb(TEXT_PRIMARY)))
                                            .cursor_pointer()
                                            .on_click(cx.listener(|this, _event, _window, cx| {
                                                this.select_all_filtered(false, cx);
                                            }))
                                            .child("Снять выбор")
                                    )
                            )
                            // Right: Ghost Sorting, Ghost Duplicate Search & Primary Red Delete Button
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .gap(px(10.))
                                    // Ghost Sort Button
                                    .child(
                                        div()
                                            .id("btn-sort-toggle")
                                            .flex()
                                            .flex_row()
                                            .items_center()
                                            .gap(px(6.))
                                            .px(px(12.))
                                            .py(px(6.))
                                            .rounded(px(8.))
                                            .border_1()
                                            .border_color(rgb(BORDER_PRIMARY))
                                            .bg(rgba(COLOR_TRANSPARENT))
                                            .hover(|s| s.bg(rgb(BG_ROW_HOVER)))
                                            .cursor_pointer()
                                            .on_click(cx.listener(|this, _event, _window, cx| {
                                                match this.sort_field {
                                                    SortField::Size => {
                                                        if this.sort_descending {
                                                            this.sort_descending = false;
                                                        } else {
                                                            this.sort_field = SortField::Date;
                                                            this.sort_descending = true;
                                                        }
                                                    }
                                                    SortField::Date => {
                                                        this.sort_field = SortField::Name;
                                                        this.sort_descending = false;
                                                    }
                                                    SortField::Name => {
                                                        this.sort_field = SortField::Size;
                                                        this.sort_descending = true;
                                                    }
                                                }
                                                this.update_filtered_cache();
                                                cx.notify();
                                            }))
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .text_color(rgb(TEXT_SECONDARY))
                                                    .child(format!(
                                                        "Сортировка: {} {}",
                                                        match self.sort_field {
                                                            SortField::Size => "размер",
                                                            SortField::Date => "дата",
                                                            SortField::Name => "имя",
                                                        },
                                                        if self.sort_descending { "↓" } else { "↑" }
                                                    ))
                                            )
                                    )
                                    // Ghost "Найти дубликаты" Button
                                    .child(
                                        div()
                                            .id("btn-find-duplicates")
                                            .flex()
                                            .flex_row()
                                            .items_center()
                                            .gap(px(6.))
                                            .px(px(12.))
                                            .py(px(6.))
                                            .rounded(px(8.))
                                            .border_1()
                                            .border_color(rgb(BORDER_PRIMARY))
                                            .bg(rgba(COLOR_TRANSPARENT))
                                            .hover(|s| s.bg(rgb(BG_ROW_HOVER)))
                                            .cursor_pointer()
                                            .on_click(cx.listener(|this, _event, _window, cx| {
                                                this.trigger_find_duplicates(cx);
                                            }))
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .font_weight(FontWeight::MEDIUM)
                                                    .text_color(rgb(TEXT_PRIMARY))
                                                    .child(if self.is_hashing { "⏳ Хеширование..." } else { "⧉ Найти дубликаты" })
                                            )
                                    )
                                    // Primary Delete Button (Warm Red #E5484D)
                                    .child(
                                        div()
                                            .id("btn-delete-selected")
                                            .flex()
                                            .flex_row()
                                            .items_center()
                                            .gap(px(6.))
                                            .px(px(14.))
                                            .py(px(6.))
                                            .rounded(px(8.))
                                            .bg(if selected_count > 0 {
                                                rgb(COLOR_RED_PRIMARY)
                                            } else {
                                                rgb(0x402322)
                                            })
                                            .hover(|s| {
                                                if selected_count > 0 {
                                                    s.bg(rgb(COLOR_RED_HOVER))
                                                } else {
                                                    s
                                                }
                                            })
                                            .cursor_pointer()
                                            .on_click(cx.listener(|this, _event, _window, cx| {
                                                if this.get_selected_stats().0 > 0 {
                                                    this.show_confirm_dialog = true;
                                                    cx.notify();
                                                }
                                            }))
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .font_weight(FontWeight::SEMIBOLD)
                                                    .text_color(if selected_count > 0 {
                                                        rgb(0xffffff)
                                                    } else {
                                                        rgb(0xa87e7e)
                                                    })
                                                    .child("🗑 Удалить выбранное")
                                            )
                                    )
                            )
                    )
                    // FILE LIST (Virtualized with display_limit to prevent UI lag and crashes)
                    .child({
                        let total_matching = self.filtered_files.len();
                        let limit = self.display_limit.min(total_matching);
                        let files_slice = &self.filtered_files[..limit];

                        let mut rendered_elements = Vec::new();
                        let mut last_group_id: Option<usize> = None;

                        if total_matching == 0 {
                            rendered_elements.push(
                                div()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .h(px(200.))
                                    .text_sm()
                                    .text_color(rgb(TEXT_MUTED))
                                    .child("Нет файлов для отображения в выбранной категории")
                                    .into_any_element()
                            );
                        } else {
                            for (idx, file) in files_slice.iter().enumerate() {
                                let id = file.id;
                                let is_selected = file.selected_for_deletion;
                                let is_dup = file.duplicate_group_id.is_some();
                                let is_orig = file.is_original;
                                let group_id = file.duplicate_group_id;

                                if let Some(gid) = group_id {
                                    if last_group_id != Some(gid) {
                                        last_group_id = Some(gid);
                                        rendered_elements.push(
                                            div()
                                                .flex()
                                                .flex_row()
                                                .items_center()
                                                .gap(px(6.))
                                                .px(px(10.))
                                                .pt(px(10.))
                                                .pb(px(4.))
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .font_weight(FontWeight::BOLD)
                                                        .text_color(rgb(TEXT_DUP_HEADER))
                                                        .child(format!(
                                                            "👥 ГРУППА ДУБЛИКАТОВ · {} — совпадение по хешу SHA-256",
                                                            file.name.to_uppercase()
                                                        ))
                                                )
                                                .into_any_element()
                                        );
                                    }
                                } else {
                                    last_group_id = None;
                                }

                                let icon_path = get_icon_path_for_file(file.category, file.extension.as_deref());

                                let subtitle = if is_dup {
                                    if is_orig {
                                        "папка Загрузки · оставить (оригинал) · нажмите, чтобы открыть".to_string()
                                    } else {
                                        "копия · хеш SHA-256 100% совпадает · нажмите, чтобы проверить".to_string()
                                    }
                                } else {
                                    format!("{} · нажмите, чтобы открыть", format_date_label(file.last_accessed, file.is_old))
                                };

                                let row = div()
                                        .id(SharedString::from(format!("file-row-{}", id)))
                                        .flex()
                                        .flex_row()
                                        .justify_between()
                                        .items_center()
                                        .px(px(10.))
                                        .py(px(6.))
                                        .rounded(px(8.))
                                        .bg(if is_dup {
                                            rgb(BG_DUP_ROW)
                                        } else {
                                            rgba(COLOR_TRANSPARENT)
                                        })
                                        .border_1()
                                        .border_color(if is_dup {
                                            rgb(0x2a201f)
                                        } else {
                                            rgba(COLOR_TRANSPARENT)
                                        })
                                        .hover(|s| {
                                            if is_dup {
                                                s.bg(rgb(0x281a19))
                                            } else {
                                                s.bg(rgb(BG_ROW_HOVER))
                                            }
                                        })
                                        // Left: Checkbox, Squircle Category Badge, Clickable Name & Subtitle
                                        .child(
                                            div()
                                                .flex()
                                                .flex_row()
                                                .items_center()
                                                .gap(px(12.))
                                                // Checkbox
                                                .child(
                                                    div()
                                                        .id(SharedString::from(format!("chk-file-{}", id)))
                                                        .w(px(18.))
                                                        .h(px(18.))
                                                        .rounded(px(5.))
                                                        .border_1()
                                                        .border_color(if is_selected {
                                                            rgb(COLOR_RED_PRIMARY)
                                                        } else {
                                                            rgb(BORDER_CHECKBOX)
                                                        })
                                                        .bg(if is_selected {
                                                            rgb(COLOR_RED_PRIMARY)
                                                        } else {
                                                            rgb(0x1a1816)
                                                        })
                                                        .cursor_pointer()
                                                        .flex()
                                                        .items_center()
                                                        .justify_center()
                                                        .text_xs()
                                                        .font_weight(FontWeight::BOLD)
                                                        .text_color(rgb(0xffffff))
                                                        .on_click(cx.listener(move |this, _event, _window, cx| {
                                                            this.toggle_file_selection(id, cx);
                                                        }))
                                                        .child(if is_selected {
                                                            div()
                                                                .child("✓")
                                                                .with_animation(
                                                                    SharedString::from(format!("chk-anim-{}", id)),
                                                                    Animation::new(Duration::from_millis(120)).with_easing(ease_in_out),
                                                                    |ch, delta| ch.opacity(delta),
                                                                )
                                                                .into_any_element()
                                                        } else {
                                                            div().into_any_element()
                                                        })
                                                )
                                                // Squircle Icon Badge with official SVG from glincker/thesvg (Click to open file in system)
                                                .child(
                                                    div()
                                                        .id(SharedString::from(format!("badge-file-{}", id)))
                                                        .w(px(32.))
                                                        .h(px(32.))
                                                        .rounded(px(8.))
                                                        .bg(rgb(0x1f1d1b))
                                                        .border_1()
                                                        .border_color(rgb(BORDER_PRIMARY))
                                                        .hover(|s| s.bg(rgb(0x282522)))
                                                        .flex()
                                                        .items_center()
                                                        .justify_center()
                                                        .cursor_pointer()
                                                        .on_click(cx.listener(move |this, _event, _window, _cx| {
                                                            this.open_file(id);
                                                        }))
                                                        .child(
                                                            img(icon_path)
                                                                .size(px(20.))
                                                        )
                                                )
                                                // Name & Subtitle (Click to open file in system viewer)
                                                .child(
                                                    div()
                                                        .id(SharedString::from(format!("title-file-{}", id)))
                                                        .flex()
                                                        .flex_col()
                                                        .cursor_pointer()
                                                        .on_click(cx.listener(move |this, _event, _window, _cx| {
                                                            this.open_file(id);
                                                        }))
                                                        .child(
                                                            div()
                                                                .text_sm()
                                                                .font_weight(FontWeight::MEDIUM)
                                                                .text_color(rgb(TEXT_PRIMARY))
                                                                .hover(|s| s.text_color(rgb(0xffffff)))
                                                                .child(file.name.clone())
                                                        )
                                                        .child(
                                                            div()
                                                                .text_xs()
                                                                .text_color(rgb(TEXT_MUTED))
                                                                .child(subtitle)
                                                        )
                                                )
                                        )
                                        // Right: "старое" pill badge, Size & Action Buttons (Open + Delete)
                                        .child(
                                            div()
                                                .flex()
                                                .flex_row()
                                                .items_center()
                                                .gap(px(12.))
                                                .children(if file.is_old && !is_dup {
                                                    vec![
                                                        div()
                                                            .px(px(8.))
                                                            .py(px(2.))
                                                            .rounded(px(5.))
                                                            .bg(rgb(BG_WARN_AMBER))
                                                            .border_1()
                                                            .border_color(rgb(BORDER_WARN_AMBER))
                                                            .text_xs()
                                                            .font_weight(FontWeight::MEDIUM)
                                                            .text_color(rgb(COLOR_WARN_AMBER))
                                                            .child("старое")
                                                            .into_any_element()
                                                    ]
                                                } else {
                                                    vec![]
                                                })
                                                .child(
                                                    div()
                                                        .text_sm()
                                                        .font_weight(FontWeight::NORMAL)
                                                        .text_color(rgb(TEXT_SECONDARY))
                                                        .child(format_bytes(file.size))
                                                )
                                                // Open in system viewer icon button
                                                .child(
                                                    div()
                                                        .id(SharedString::from(format!("open-btn-{}", id)))
                                                        .w(px(22.))
                                                        .h(px(22.))
                                                        .cursor_pointer()
                                                        .flex()
                                                        .items_center()
                                                        .justify_center()
                                                        .text_xs()
                                                        .text_color(rgb(TEXT_MUTED))
                                                        .hover(|s| s.text_color(rgb(TEXT_PRIMARY)))
                                                        .on_click(cx.listener(move |this, _event, _window, _cx| {
                                                            this.open_file(id);
                                                        }))
                                                        .child("↗")
                                                )
                                                // Quick delete icon button
                                                .child(
                                                    div()
                                                        .id(SharedString::from(format!("del-file-{}", id)))
                                                        .w(px(20.))
                                                        .h(px(20.))
                                                        .cursor_pointer()
                                                        .flex()
                                                        .items_center()
                                                        .justify_center()
                                                        .text_xs()
                                                        .text_color(rgb(TEXT_MUTED))
                                                        .hover(|s| s.text_color(rgb(COLOR_RED_PRIMARY)))
                                                        .on_click(cx.listener(move |this, _event, _window, cx| {
                                                            this.delete_single_file(id, cx);
                                                        }))
                                                        .child("✕")
                                                )
                                        );

                                if idx < 15 {
                                    rendered_elements.push(
                                        row.with_animation(
                                            SharedString::from(format!("row-anim-{}-{}", self.filter_version, id)),
                                            Animation::new(Duration::from_millis(70 + (idx as u64 * 6).min(100))).with_easing(ease_in_out),
                                            |r, delta| r.opacity(delta).mt(px((1.0 - delta) * 2.0)),
                                        ).into_any_element()
                                    );
                                } else {
                                    rendered_elements.push(row.into_any_element());
                                }
                            }

                            // Show "Load More" button if items exceed limit to prevent UI lag
                            if total_matching > limit {
                                rendered_elements.push(
                                    div()
                                        .id("btn-load-more")
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .w_full()
                                        .py(px(10.))
                                        .my(px(6.))
                                        .rounded(px(8.))
                                        .border_1()
                                        .border_color(rgb(BORDER_PRIMARY))
                                        .bg(rgb(BG_ROW_HOVER))
                                        .hover(|s| s.bg(rgb(BG_SIDEBAR_ACTIVE)))
                                        .cursor_pointer()
                                        .on_click(cx.listener(|this, _event, _window, cx| {
                                            this.display_limit += 100;
                                            cx.notify();
                                        }))
                                        .child(
                                            div()
                                                .text_xs()
                                                .font_weight(FontWeight::MEDIUM)
                                                .text_color(rgb(TEXT_SECONDARY))
                                                .child(format!(
                                                    "Показать ещё 100 файлов (показано {} из {})",
                                                    limit, total_matching
                                                ))
                                        )
                                        .into_any_element()
                                );
                            }
                        }

                        let list_container = div()
                            .id("main-file-list-scroll")
                            .flex()
                            .flex_col()
                            .flex_1()
                            .min_w_0()
                            .gap(px(2.))
                            .children(rendered_elements);

                        // Disable scroll bubbling when confirmation modal is active
                        if is_modal_open {
                            list_container.overflow_hidden()
                        } else {
                            list_container.overflow_y_scroll()
                        }
                    })
                    // BOTTOM STATUS SUMMARY
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .justify_between()
                            .items_center()
                            .pt(px(4.))
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .text_xs()
                                    .child(
                                        div()
                                            .text_color(rgb(TEXT_SECONDARY))
                                            .child("Можно освободить до ")
                                    )
                                    .child(
                                        div()
                                            .font_weight(FontWeight::BOLD)
                                            .text_color(rgb(TEXT_PRIMARY))
                                            .child(format_bytes(self.summary.old_files_total_size))
                                    )
                                    .child(
                                        div()
                                            .text_color(rgb(TEXT_SECONDARY))
                                            .child(", удалив старые файлы")
                                    )
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(TEXT_MUTED))
                                    .child(format!(
                                        "{} файлов · {} всего",
                                        self.summary.total_files,
                                        format_bytes(self.summary.total_size)
                                    ))
                            )
                    )
            )
            // -------------------------------------------------------------
            // CONFIRMATION MODAL (Mass Deletion to Trash)
            // -------------------------------------------------------------
            .children(if self.show_confirm_dialog {
                let selected_files: Vec<&FileItem> = self
                    .files
                    .iter()
                    .filter(|f| f.selected_for_deletion)
                    .collect();

                vec![
                    div()
                        .id("modal-backdrop")
                        .absolute()
                        .size_full()
                        .bg(rgba(0x000000cc))
                        .flex()
                        .items_center()
                        .justify_center()
                        // Intercept clicks on backdrop
                        .on_click(|_event, _window, _cx| {})
                        .child(
                            div()
                                .id("modal-card")
                                .flex()
                                .flex_col()
                                .w(px(520.))
                                .bg(rgb(BG_MODAL))
                                .border_1()
                                .border_color(rgb(BORDER_PRIMARY))
                                .rounded(px(14.))
                                .p(px(24.))
                                .gap(px(16.))
                                .child(
                                    div()
                                        .text_base()
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(rgb(TEXT_PRIMARY))
                                        .child("Подтверждение перемещения в Корзину")
                                )
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(rgb(TEXT_SECONDARY))
                                        .child(format!(
                                            "Вы собираетесь переместить в системную Корзину {} файлов общим размером {}. Файлы не удаляются насовсем и могут быть восстановлены из Корзины.",
                                            selected_count,
                                            format_bytes(selected_size)
                                        ))
                                )
                                // Scrollable Preview of Files (Isolated inside modal)
                                .child(
                                    div()
                                        .id("modal-files-preview-scroll")
                                        .h(px(180.))
                                        .overflow_y_scroll()
                                        .bg(rgb(BG_WINDOW))
                                        .border_1()
                                        .border_color(rgb(BORDER_SUBTLE))
                                        .rounded(px(8.))
                                        .p(px(10.))
                                        .gap(px(4.))
                                        .children(selected_files.iter().map(|&f| {
                                            let icon_p = get_icon_path_for_file(f.category, f.extension.as_deref());
                                            div()
                                                .flex()
                                                .flex_row()
                                                .justify_between()
                                                .items_center()
                                                .text_xs()
                                                .child(
                                                    div()
                                                        .flex()
                                                        .flex_row()
                                                        .items_center()
                                                        .gap(px(8.))
                                                        .child(img(icon_p).size(px(14.)))
                                                        .child(div().text_color(rgb(TEXT_PRIMARY)).child(f.name.clone()))
                                                )
                                                .child(div().text_color(rgb(TEXT_MUTED)).child(format_bytes(f.size)))
                                        }))
                                )
                                // Action Buttons
                                .child(
                                    div()
                                        .flex()
                                        .flex_row()
                                        .justify_end()
                                        .gap(px(10.))
                                        .child(
                                            div()
                                                .id("btn-modal-cancel")
                                                .px(px(14.))
                                                .py(px(7.))
                                                .rounded(px(8.))
                                                .border_1()
                                                .border_color(rgb(BORDER_PRIMARY))
                                                .bg(rgba(COLOR_TRANSPARENT))
                                                .hover(|s| s.bg(rgb(BG_ROW_HOVER)))
                                                .cursor_pointer()
                                                .on_click(cx.listener(|this, _event, _window, cx| {
                                                    this.show_confirm_dialog = false;
                                                    cx.notify();
                                                }))
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .font_weight(FontWeight::MEDIUM)
                                                        .text_color(rgb(TEXT_SECONDARY))
                                                        .child("Отмена")
                                                )
                                        )
                                        .child(
                                            div()
                                                .id("btn-modal-confirm")
                                                .px(px(14.))
                                                .py(px(7.))
                                                .rounded(px(8.))
                                                .bg(rgb(COLOR_RED_PRIMARY))
                                                .hover(|s| s.bg(rgb(COLOR_RED_HOVER)))
                                                .cursor_pointer()
                                                .on_click(cx.listener(|this, _event, _window, cx| {
                                                    this.execute_mass_trash(cx);
                                                }))
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .font_weight(FontWeight::BOLD)
                                                        .text_color(rgb(0xffffff))
                                                        .child("🗑 Переместить в корзину")
                                                )
                                        )
                                )
                        )
                        .into_any_element()
                ]
            } else {
                vec![]
            })
            // -------------------------------------------------------------
            // UNDO TOAST NOTIFICATION (Temporary 6-8 seconds)
            // -------------------------------------------------------------
            .children(if let Some(undo) = &self.undo_state {
                let undo_count = undo.count;
                let undo_size = undo.total_size;
                let undo_id = undo.id;

                vec![
                    div()
                        .id(SharedString::from(format!("toast-overlay-{}", undo_id)))
                        .absolute()
                        .bottom(px(24.))
                        .left_0()
                        .right_0()
                        .flex()
                        .justify_center()
                        .items_center()
                        .child(
                            div()
                                .id("undo-toast-card")
                                .flex()
                                .flex_row()
                                .items_center()
                                .gap(px(14.))
                                .bg(rgb(0x1f1d1b))
                                .border_1()
                                .border_color(rgb(0x38342e))
                                .rounded(px(10.))
                                .px(px(18.))
                                .py(px(10.))
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(rgb(TEXT_PRIMARY))
                                        .child(format!("Удалено {} файлов ({})", undo_count, format_bytes(undo_size)))
                                )
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(rgb(TEXT_MUTED))
                                        .child("·")
                                )
                                .child(
                                    div()
                                        .id("btn-undo")
                                        .text_sm()
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(rgb(0xffffff))
                                        .hover(|s| s.underline().text_color(rgb(COLOR_WARN_AMBER)))
                                        .cursor_pointer()
                                        .on_click(cx.listener(|this, _event, _window, cx| {
                                            this.trigger_undo(cx);
                                        }))
                                        .child("Отменить")
                                )
                                .child(
                                    div()
                                        .id("btn-dismiss-undo")
                                        .text_xs()
                                        .text_color(rgb(TEXT_MUTED))
                                        .hover(|s| s.text_color(rgb(TEXT_PRIMARY)))
                                        .cursor_pointer()
                                        .on_click(cx.listener(|this, _event, _window, cx| {
                                            this.undo_state = None;
                                            cx.notify();
                                        }))
                                        .child("✕")
                                )
                        )
                        .with_animation(
                            SharedString::from(format!("toast-anim-{}", undo_id)),
                            Animation::new(Duration::from_millis(220)).with_easing(ease_in_out),
                            |t, delta| t.opacity(delta).bottom(px(14.0 + delta * 10.0)),
                        )
                        .into_any_element()
                ]
            } else {
                vec![]
            })
    }
}

#[cfg(windows)]
mod smooth_drag;

fn main() {
    #[cfg(windows)]
    smooth_drag::spawn_smooth_drag_monitor();

    Application::new()
        .with_assets(AppAssets)
        .run(|cx: &mut App| {
            cx.bind_keys([
                KeyBinding::new("f11", ToggleFullscreen, None),
                KeyBinding::new("alt-enter", ToggleFullscreen, None),
            ]);

            let window_options = WindowOptions {
                titlebar: Some(TitlebarOptions {
                    title: Some("DownSort".into()),
                    appears_transparent: true,
                    traffic_light_position: None,
                }),
                window_bounds: Some(WindowBounds::Windowed(Bounds {
                    origin: Point::new(px(100.), px(100.)),
                    size: Size {
                        width: px(1160.),
                        height: px(760.),
                    },
                })),
                is_movable: true,
                is_resizable: true,
                is_minimizable: true,
                ..Default::default()
            };

            cx.open_window(window_options, |_, cx| {
                cx.new(|cx| DownSortApp::new(cx))
            })
            .expect("Не удалось создать окно DownSort");
        });
}
