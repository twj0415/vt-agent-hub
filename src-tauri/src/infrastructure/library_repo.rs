use std::fs;
use std::path::{Path, PathBuf};

use crate::core::library_layout::LIBRARY_AREAS;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LibraryStructureItem {
    pub key: String,
    pub path: String,
    pub kind: String,
    pub status: String,
    pub repair_hint: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LibraryEnsureResult {
    pub created_paths: Vec<String>,
    pub existing_paths: Vec<String>,
}

pub struct LibraryRepo;

impl LibraryRepo {
    pub fn ensure(root: &Path) -> Result<LibraryEnsureResult, String> {
        let mut created_paths = Vec::new();
        let mut existing_paths = Vec::new();

        Self::ensure_dir(root, &mut created_paths, &mut existing_paths)?;
        if !root.is_dir() {
            return Ok(LibraryEnsureResult {
                created_paths,
                existing_paths,
            });
        }

        for area in LIBRARY_AREAS {
            Self::ensure_dir(
                &root.join(area.relative_path),
                &mut created_paths,
                &mut existing_paths,
            )?;
        }

        Ok(LibraryEnsureResult {
            created_paths,
            existing_paths,
        })
    }

    pub fn inspect(root: &Path) -> Vec<LibraryStructureItem> {
        let mut items = vec![Self::inspect_dir("library_root", root)];
        for area in LIBRARY_AREAS {
            items.push(Self::inspect_dir(area.key, &root.join(area.relative_path)));
        }
        items
    }

    fn ensure_dir(
        path: &Path,
        created_paths: &mut Vec<String>,
        existing_paths: &mut Vec<String>,
    ) -> Result<(), String> {
        if path.exists() {
            if path.is_dir() {
                existing_paths.push(path.display().to_string());
            }
            return Ok(());
        }

        fs::create_dir_all(path).map_err(|error| {
            format!(
                "Failed to create library directory {}: {error}",
                path.display()
            )
        })?;
        created_paths.push(path.display().to_string());
        Ok(())
    }

    fn inspect_dir(key: &str, path: &Path) -> LibraryStructureItem {
        let (status, repair_hint) = if !path.exists() {
            (
                "missing".to_string(),
                "Run library ensure to create this directory.".to_string(),
            )
        } else if !path.is_dir() {
            (
                "invalid".to_string(),
                "Path exists but is not a directory; manual repair is required.".to_string(),
            )
        } else {
            ("ok".to_string(), "No repair needed.".to_string())
        };

        LibraryStructureItem {
            key: key.to_string(),
            path: path.display().to_string(),
            kind: "directory".to_string(),
            status,
            repair_hint,
        }
    }
}

pub fn library_area_path(root: &Path, key: &str) -> Option<PathBuf> {
    LIBRARY_AREAS
        .iter()
        .find(|area| area.key == key)
        .map(|area| root.join(area.relative_path))
}
