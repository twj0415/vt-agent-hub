#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LibraryArea {
    pub key: &'static str,
    pub relative_path: &'static str,
    pub responsibility: &'static str,
}

pub const LIBRARY_AREAS: [LibraryArea; 1] = [LibraryArea {
    key: "skills",
    relative_path: "skills",
    responsibility:
        "Central skill source directories copied into tool runtime directories on install.",
}];

pub const PROJECT_OUTPUT_NOTE: &str =
    "Project output is resolved by the active ToolAdapter and remains in the target project directory.";

pub fn library_area(key: &str) -> Option<LibraryArea> {
    LIBRARY_AREAS.iter().find(|area| area.key == key).copied()
}

#[cfg(test)]
mod tests {
    use super::{library_area, LIBRARY_AREAS, PROJECT_OUTPUT_NOTE};

    #[test]
    fn defines_required_library_areas() {
        let keys = LIBRARY_AREAS
            .iter()
            .map(|area| area.key)
            .collect::<Vec<_>>();

        assert_eq!(keys, vec!["skills"]);
        assert_eq!(library_area("skills").unwrap().relative_path, "skills");
        assert!(PROJECT_OUTPUT_NOTE.contains("ToolAdapter"));
    }
}
