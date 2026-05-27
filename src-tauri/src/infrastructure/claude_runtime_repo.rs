use std::path::PathBuf;

use crate::core::paths;

pub struct ClaudeRuntimeRepo;

impl ClaudeRuntimeRepo {
    pub fn root() -> PathBuf {
        paths::claude_root()
    }

    pub fn global_claude_path() -> PathBuf {
        Self::root().join("CLAUDE.md")
    }
}
