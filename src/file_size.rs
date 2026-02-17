use std::fmt;

pub struct FileSize(pub u64);

impl fmt::Display for FileSize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let bytes = self.0 as f64;

        const KB: f64 = 1024.0;
        const MB: f64 = KB * 1024.0;
        const GB: f64 = MB * 1024.0;
        const TB: f64 = GB * 1024.0;

        if bytes >= TB {
            write!(f, "{:.1}TB", bytes / TB)
        } else if bytes >= GB {
            write!(f, "{:.1}GB", bytes / GB)
        } else if bytes >= MB {
            write!(f, "{:.1}MB", bytes / MB)
        } else if bytes >= KB {
            write!(f, "{:.1}KB", bytes / KB)
        } else {
            write!(f, "{}B", self.0)
        }
    }
}
