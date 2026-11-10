use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum FileCategory {
    Documents,
    Pdf,
    Spreadsheets,
    Photos,
    Archives,
    Installers,
    Media,
    Other,
}

impl FileCategory {
    /// Categorize file by its extension (case-insensitive).
    pub fn from_extension(ext: &str) -> Self {
        let ext_lower = ext.trim_start_matches('.').to_lowercase();
        match ext_lower.as_str() {
            // Документы: doc, docx, odt, rtf
            "doc" | "docx" | "odt" | "rtf" => FileCategory::Documents,

            // PDF: pdf
            "pdf" => FileCategory::Pdf,

            // Таблицы: xls, xlsx, csv, ods
            "xls" | "xlsx" | "csv" | "ods" => FileCategory::Spreadsheets,

            // Фото: jpg, jpeg, png, webp, heic, gif, bmp
            "jpg" | "jpeg" | "png" | "webp" | "heic" | "gif" | "bmp" => FileCategory::Photos,

            // Архивы: zip, rar, 7z, tar, gz
            "zip" | "rar" | "7z" | "tar" | "gz" => FileCategory::Archives,

            // Установщики: exe, msi, deb, rpm, appimage, dmg
            "exe" | "msi" | "deb" | "rpm" | "appimage" | "dmg" => FileCategory::Installers,

            // Видео/Аудио: mp4, mkv, avi, mov, mp3, wav, flac
            "mp4" | "mkv" | "avi" | "mov" | "mp3" | "wav" | "flac" => FileCategory::Media,

            // Прочее: всё остальное
            _ => FileCategory::Other,
        }
    }

    /// Russian display name for UI
    pub fn display_name(&self) -> &'static str {
        match self {
            FileCategory::Documents => "Документы",
            FileCategory::Pdf => "PDF",
            FileCategory::Spreadsheets => "Таблицы",
            FileCategory::Photos => "Фото",
            FileCategory::Archives => "Архивы",
            FileCategory::Installers => "Установщики",
            FileCategory::Media => "Видео/Аудио",
            FileCategory::Other => "Прочее",
        }
    }

    /// Color hex code for UI category pills and donut chart (warm toned down to match design/ref.png)
    pub fn color_hex(&self) -> &'static str {
        match self {
            FileCategory::Documents => "#4F85E0",    // Blue
            FileCategory::Pdf => "#FF6B57",          // Coral Red
            FileCategory::Spreadsheets => "#38C793", // Mint Teal
            FileCategory::Photos => "#38C793",       // Emerald Green (IMG in reference)
            FileCategory::Archives => "#F0A21C",     // Warm Amber (ZIP in reference)
            FileCategory::Installers => "#C58CF0",   // Purple (EXE in reference)
            FileCategory::Media => "#4FD1E0",        // Sky Cyan (MP4 in reference)
            FileCategory::Other => "#716A5F",        // Warm Grey
        }
    }

    /// RGB components as (u8, u8, u8)
    pub fn rgb(&self) -> (u8, u8, u8) {
        match self {
            FileCategory::Documents => (79, 133, 224),
            FileCategory::Pdf => (255, 107, 87),
            FileCategory::Spreadsheets => (56, 199, 147),
            FileCategory::Photos => (56, 199, 147),
            FileCategory::Archives => (240, 162, 28),
            FileCategory::Installers => (197, 140, 240),
            FileCategory::Media => (79, 209, 224),
            FileCategory::Other => (113, 106, 95),
        }
    }

    /// Short 3-4 letter badge abbreviation (e.g. DOC, PDF, IMG, ZIP, EXE, MP4)
    pub fn default_badge(&self) -> &'static str {
        match self {
            FileCategory::Documents => "DOC",
            FileCategory::Pdf => "PDF",
            FileCategory::Spreadsheets => "XLS",
            FileCategory::Photos => "IMG",
            FileCategory::Archives => "ZIP",
            FileCategory::Installers => "EXE",
            FileCategory::Media => "MP4",
            FileCategory::Other => "ETC",
        }
    }

    /// List of all primary categories in order matching stats indexing
    pub fn all() -> &'static [FileCategory] {
        &[
            FileCategory::Documents,
            FileCategory::Pdf,
            FileCategory::Spreadsheets,
            FileCategory::Photos,
            FileCategory::Archives,
            FileCategory::Installers,
            FileCategory::Media,
            FileCategory::Other,
        ]
    }
}

impl fmt::Display for FileCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_name())
    }
}
