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
            Self::Added => "\x1b[32m",
            Self::Modified => "\x1b[33m",
            Self::Deleted => "\x1b[31m",
            Self::Renamed => "\x1b[36m",
            Self::Untracked => "\x1b[90m",
        }
    }

    pub fn icon(&self) -> &str {
        match self {
            Self::Added => "+",
            Self::Modified => "~",
            Self::Deleted => "✕",
            Self::Renamed => "→",
            Self::Untracked => "?",
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
