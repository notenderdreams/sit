use crate::style::{
    CYAN, GREEN, GREY, ICON_ADDED, ICON_DELETED, ICON_MODIFIED, ICON_RENAMED, ICON_UNTRACKED, RED,
    YELLOW,
};

#[derive(Debug, Clone)]
pub struct FileChange {
    pub path: String,
    pub status: FileStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileStatus {
    Added,
    Modified,
    Deleted,
    Renamed,
    Untracked,
}

impl FileStatus {
    pub fn color_code(&self) -> &str {
        match self {
            Self::Added => GREEN,
            Self::Modified => YELLOW,
            Self::Deleted => RED,
            Self::Renamed => CYAN,
            Self::Untracked => GREY,
        }
    }

    pub fn icon(&self) -> &str {
        match self {
            Self::Added => ICON_ADDED,
            Self::Modified => ICON_MODIFIED,
            Self::Deleted => ICON_DELETED,
            Self::Renamed => ICON_RENAMED,
            Self::Untracked => ICON_UNTRACKED,
        }
    }

    pub fn order(&self) -> u8 {
        match self {
            Self::Modified => 0,
            Self::Added => 1,
            Self::Deleted => 2,
            Self::Renamed => 3,
            Self::Untracked => 4,
        }
    }
}

pub struct PushResult {
    pub remote: String,
    pub branch: String,
    pub set_upstream: bool,
}

#[derive(Debug, Clone)]
pub struct Branch {
    pub name: String,
    pub is_current: bool,
}
