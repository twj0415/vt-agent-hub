#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TruthSourceKind {
    Sqlite,
    FileSystem,
    SecureStorage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TruthSourceBoundary {
    pub key: &'static str,
    pub canonical: TruthSourceKind,
    pub mirrors: &'static [TruthSourceKind],
    pub note: &'static str,
}

pub const TRUTH_SOURCE_BOUNDARIES: [TruthSourceBoundary; 6] = [
    TruthSourceBoundary {
        key: "project_entities",
        canonical: TruthSourceKind::Sqlite,
        mirrors: &[TruthSourceKind::FileSystem],
        note: "Projects, rule bindings, and project metadata are owned by SQLite and may generate file output.",
    },
    TruthSourceBoundary {
        key: "catalog_assets",
        canonical: TruthSourceKind::Sqlite,
        mirrors: &[TruthSourceKind::FileSystem],
        note: "Rules and skills are managed in SQLite first and may later sync to filesystem-managed assets.",
    },
    TruthSourceBoundary {
        key: "project_output",
        canonical: TruthSourceKind::FileSystem,
        mirrors: &[TruthSourceKind::Sqlite],
        note: "Generated AGENTS.md content is owned by the filesystem and only summarized into SQLite/history after write.",
    },
    TruthSourceBoundary {
        key: "tool_runtime",
        canonical: TruthSourceKind::FileSystem,
        mirrors: &[TruthSourceKind::Sqlite],
        note: "Tool live config and runtime skill installation status are detected from real filesystem state.",
    },
    TruthSourceBoundary {
        key: "credentials",
        canonical: TruthSourceKind::SecureStorage,
        mirrors: &[TruthSourceKind::FileSystem, TruthSourceKind::Sqlite],
        note: "Secrets should live in secure storage; file/DB state may only hold presence or masked summaries.",
    },
    TruthSourceBoundary {
        key: "operation_history",
        canonical: TruthSourceKind::Sqlite,
        mirrors: &[TruthSourceKind::FileSystem],
        note: "Operations are recorded in SQLite and may reference backup/report files on disk.",
    },
];
