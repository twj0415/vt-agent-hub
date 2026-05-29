#[macro_use]
mod macros;
mod bootstrap;
mod credentials;
mod diagnostics;
mod dialogs;
mod first_run_import;
mod history_log;
mod imports;
mod project_import;
mod projects;
mod providers;
mod rules;
mod skills;
mod snapshots;
mod tools;

pub use bootstrap::{get_app_bootstrap, reset_app_data};
pub use credentials::{clear_tool_credential_state, save_tool_credential_state};
pub use diagnostics::{
    delete_backup, export_library_diagnostics, get_backup_snapshot, preview_backup_restore,
    restore_backup, scan_library_diagnostics,
};
pub use dialogs::{pick_file_path, pick_folder_path};
pub use first_run_import::{
    apply_first_run_import, dismiss_first_run_import, get_first_run_import_status,
    preview_first_run_import, reset_first_run_import_status,
};
pub use imports::{
    apply_repository_import, import_github_repo_skills, preview_github_repo_import,
    preview_repository_import,
};
pub use project_import::import_project_from_git;
pub use projects::{
    apply_global_output, apply_project_output, cleanup_global_output, cleanup_project_output,
    delete_project_entity, preview_global_output, preview_project_output, repair_global_output,
    repair_project_output, reset_project_output, save_project_entity, save_project_rule_bindings,
    save_tool_global_rule_bindings, save_tool_skill_bindings, scan_project_output,
};
pub use providers::{
    apply_provider_to_live_config, delete_provider, detect_provider_live_drift, duplicate_provider,
    import_provider_config, list_providers, preview_provider_apply, save_provider,
};
pub use rules::{
    delete_rule_asset, import_rule_asset, move_rule_asset, preview_rule_impact,
    preview_rule_import, save_rule_asset,
};
pub use skills::{
    delete_skill_asset, install_skill_asset, mark_skill_asset_stale, repair_skill_asset,
    save_skill_asset, uninstall_skill_asset,
};
pub use snapshots::{
    get_catalog_snapshot, get_history_snapshot, get_library_snapshot, get_project_context_snapshot,
    get_project_detail, get_settings_snapshot, get_tools_snapshot, get_workspace_snapshot,
};
pub use tools::{get_tool_diagnostics, repair_tool, set_tool_enabled, verify_tool_credential};
