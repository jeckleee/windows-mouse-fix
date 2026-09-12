use std::{
    fs::{File, OpenOptions},
    sync::OnceLock,
};

struct MappedFont {
    // Keep the file open so Windows prevents writes while the font is mapped.
    _file: File,
    data: memmap2::Mmap,
}

pub fn chinese_font() -> Option<&'static [u8]> {
    // egui's owned font path clones the entire file. A process-lifetime mapping lets
    // its font parser borrow the same bytes and fault in only the pages it uses.
    static FONT: OnceLock<Option<MappedFont>> = OnceLock::new();
    FONT.get_or_init(|| {
        let windows_fonts =
            std::env::var("WINDIR").unwrap_or_else(|_| "C:\\Windows".into()) + "\\Fonts\\";
        let candidates = [
            windows_fonts.clone() + "msyh.ttc",
            windows_fonts.clone() + "msjh.ttc",
            windows_fonts + "simsun.ttc",
            "/System/Library/Fonts/PingFang.ttc".into(),
            "/System/Library/Fonts/STHeiti Light.ttc".into(),
        ];
        for path in candidates {
            let mut options = OpenOptions::new();
            options.read(true);
            #[cfg(windows)]
            {
                use std::os::windows::fs::OpenOptionsExt;
                use windows_sys::Win32::Storage::FileSystem::FILE_SHARE_READ;
                options.share_mode(FILE_SHARE_READ);
            }
            if let Ok(file) = options.open(path) {
                // SAFETY: Windows denies concurrent writes/deletion via the retained
                // file handle. macOS fallback fonts reside in protected /System files.
                // FONT retains the mapping for every borrowed egui font reference.
                if let Ok(data) = unsafe { memmap2::Mmap::map(&file) } {
                    return Some(MappedFont { _file: file, data });
                }
            }
        }
        None
    })
    .as_ref()
    .map(|font| font.data.as_ref())
}
