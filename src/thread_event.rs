use std::path::PathBuf;

#[derive(Debug)]
pub enum LogicEvent {
    FolderSize(PathBuf),
}

#[derive(Debug)]
pub enum RenderEvent {
    FolderSize(u64),
}
