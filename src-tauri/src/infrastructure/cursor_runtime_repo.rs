use std::path::PathBuf;

use crate::core::paths;

pub struct CursorRuntimeRepo;

impl CursorRuntimeRepo {
    pub fn root() -> PathBuf {
        paths::cursor_root()
    }

    pub fn global_rules_root() -> PathBuf {
        Self::root().join("rules")
    }
}
