pub struct FileSize(pub u64);

impl ToString for FileSize {
    fn to_string(&self) -> String {
        let bytes = self.0 as f64;

        const KB: f64 = 1024.0;
        const MB: f64 = KB * 1024.0;
        const GB: f64 = MB * 1024.0;
        const TB: f64 = GB * 1024.0;

        if bytes >= TB {
            format!("{:.1}TB", bytes / TB)
        } else if bytes >= GB {
            format!("{:.1}GB", bytes / GB)
        } else if bytes >= MB {
            format!("{:.1}MB", bytes / MB)
        } else if bytes >= KB {
            format!("{:.1}KB", bytes / KB)
        } else {
            format!("{}B", self.0)
        }
    }
}
