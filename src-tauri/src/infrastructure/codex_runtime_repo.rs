use std::path::PathBuf;

use crate::adapters::tool_adapter::ToolActionResult;
use crate::core::paths;
use crate::core::status_codes::{
    SKILL_INSTALL_INSTALLED, SKILL_INSTALL_NOT_INSTALLED, SKILL_INSTALL_SOURCE_MISSING,
    TARGET_STATE_ERROR, TARGET_STATE_MISSING, TARGET_STATE_PLANNED, TARGET_STATE_READY,
};

pub struct CodexRuntimeRepo;

impl CodexRuntimeRepo {
    pub fn codex_root() -> PathBuf {
        paths::codex_root()
    }

    pub fn config_path() -> PathBuf {
        Self::codex_root().join("config.toml")
    }

    pub fn version_path() -> PathBuf {
        Self::codex_root().join("version.json")
    }

    pub fn global_agents_path() -> PathBuf {
        Self::codex_root().join("AGENTS.md")
    }

    pub fn skills_path() -> PathBuf {
        Self::codex_root().join("skills")
    }

    pub fn detect_installation() -> bool {
        Self::codex_root().exists()
    }

    pub fn version() -> String {
        let Ok(content) = std::fs::read_to_string(Self::version_path()) else {
            return "-".to_string();
        };
        serde_json::from_str::<serde_json::Value>(&content)
            .ok()
            .and_then(|value| {
                value
                    .get("latest_version")
                    .and_then(|version| version.as_str())
                    .map(str::to_string)
            })
            .unwrap_or_else(|| "-".to_string())
    }

    pub fn credential_state() -> String {
        if Self::config_path().exists() {
            "config_present".to_string()
        } else {
            "config_missing".to_string()
        }
    }

    pub fn credential_state_code() -> i32 {
        if Self::config_path().exists() {
            TARGET_STATE_READY
        } else {
            TARGET_STATE_MISSING
        }
    }

    pub fn skill_state() -> String {
        if Self::skills_path().exists() {
            "installed".to_string()
        } else if Self::detect_installation() {
            "not_installed".to_string()
        } else {
            "source_missing".to_string()
        }
    }

    pub fn skill_state_code() -> i32 {
        if Self::skills_path().exists() {
            SKILL_INSTALL_INSTALLED
        } else if Self::detect_installation() {
            SKILL_INSTALL_NOT_INSTALLED
        } else {
            SKILL_INSTALL_SOURCE_MISSING
        }
    }

    pub fn project_output_state() -> String {
        if Self::config_path().exists() {
            "preview_ready".to_string()
        } else {
            "config_missing".to_string()
        }
    }

    pub fn project_output_state_code() -> i32 {
        if Self::config_path().exists() {
            TARGET_STATE_READY
        } else {
            TARGET_STATE_MISSING
        }
    }

    pub fn repair_state() -> String {
        if !Self::detect_installation() {
            "blocked".to_string()
        } else if Self::config_path().exists() {
            "ready".to_string()
        } else {
            "manual_required".to_string()
        }
    }

    pub fn repair_state_code() -> i32 {
        if !Self::detect_installation() {
            TARGET_STATE_ERROR
        } else if Self::config_path().exists() {
            TARGET_STATE_READY
        } else {
            TARGET_STATE_PLANNED
        }
    }

    pub fn repair_hint() -> String {
        if !Self::detect_installation() {
            format!(
                "Install Codex first, then confirm the runtime root exists at {}.",
                Self::codex_root().display()
            )
        } else if Self::config_path().exists() {
            "Tool environment is ready. Project output repair stays in the Projects page and only touches project AGENTS.md files.".to_string()
        } else {
            format!(
                "Create or restore {}, then run Check connection again.",
                Self::config_path().display()
            )
        }
    }

    pub fn verify_credential(token: &str) -> ToolActionResult {
        let trimmed = token.trim();

        if trimmed.is_empty() {
            return ToolActionResult {
                ok: false,
                state: "local_invalid".to_string(),
                detail:
                    "Credential is empty. Enter the Codex credential before checking connection."
                        .to_string(),
                manual_steps: vec!["Paste the credential into the access token field.".to_string()],
            };
        }

        if trimmed.len() < 8 || trimmed.contains(char::is_whitespace) {
            return ToolActionResult {
                ok: false,
                state: "local_invalid".to_string(),
                detail: "Credential failed local format validation. It must be at least 8 non-whitespace characters.".to_string(),
                manual_steps: vec![
                    "Check that the token was copied completely.".to_string(),
                    "Remove spaces, line breaks, or surrounding quotes.".to_string(),
                ],
            };
        }

        if !Self::detect_installation() {
            return ToolActionResult {
                ok: false,
                state: "local_valid_remote_unavailable".to_string(),
                detail: format!(
                    "Credential format is valid, but remote verification is blocked because Codex root is missing at {}.",
                    Self::codex_root().display()
                ),
                manual_steps: vec![
                    "Install or initialize Codex on this machine.".to_string(),
                    format!("Confirm {} exists.", Self::codex_root().display()),
                    "Run Check connection again after Codex is available.".to_string(),
                ],
            };
        }

        ToolActionResult {
            ok: false,
            state: "local_valid_remote_unavailable".to_string(),
            detail: "Credential passed local validation. V1 cannot safely verify a remote provider without a configured verification endpoint, so it does not report a false success.".to_string(),
            manual_steps: vec![
                format!("Confirm Codex can read {}.", Self::config_path().display()),
                "Run a real Codex command outside VT Hub Manager if you need provider-level confirmation.".to_string(),
                "If that command succeeds, the credential is usable for Codex.".to_string(),
            ],
        }
    }

    pub fn repair() -> ToolActionResult {
        if !Self::detect_installation() {
            return ToolActionResult {
                ok: false,
                state: "tool_environment_manual_required".to_string(),
                detail:
                    "Tool environment repair is blocked because the Codex runtime root is missing."
                        .to_string(),
                manual_steps: vec![
                    "Install or initialize Codex on this machine.".to_string(),
                    format!("Confirm {} exists.", Self::codex_root().display()),
                    "Return to VT Hub Manager and run Repair again.".to_string(),
                ],
            };
        }

        if Self::config_path().exists() {
            return ToolActionResult {
                ok: true,
                state: "tool_environment_ready".to_string(),
                detail: "Tool environment repair is not needed. Codex config already exists.".to_string(),
                manual_steps: vec![
                    "For project output repair, open Projects and use Repair on the affected project card.".to_string(),
                    "Tool repair only checks the Codex environment; it does not rewrite project AGENTS.md files.".to_string(),
                ],
            };
        }

        ToolActionResult {
            ok: false,
            state: "tool_environment_manual_required".to_string(),
            detail: format!(
                "Codex runtime exists, but {} is missing. VT Hub Manager will not invent a config file because that could create invalid provider settings.",
                Self::config_path().display()
            ),
            manual_steps: vec![
                format!("Restore or create {}.", Self::config_path().display()),
                "Keep provider credentials in Codex's expected format.".to_string(),
                "Run Check connection again after the config is restored.".to_string(),
            ],
        }
    }
}
