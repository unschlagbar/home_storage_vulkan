use std::path::PathBuf;

pub enum LogicEvent {
    FolderSize(PathBuf),
}

pub enum RenderEvent {
    FolderSize(usize),
}
