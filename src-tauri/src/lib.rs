pub mod adapters;
pub mod application;
pub mod commands;
pub mod core;
pub mod domain;
pub mod dto;
pub mod infrastructure;

use application::app_container::AppContainer;
use commands::{
    apply_first_run_import, apply_global_output, apply_project_output,
    apply_provider_to_live_config, apply_repository_import, cleanup_global_output,
    cleanup_project_output, clear_tool_credential_state, delete_backup, delete_project_entity,
    delete_provider, delete_rule_asset, delete_skill_asset, detect_provider_live_drift,
    dismiss_first_run_import, duplicate_provider, export_library_diagnostics, get_app_bootstrap, get_backup_snapshot,
    get_catalog_snapshot, get_first_run_import_status, get_history_snapshot, get_library_snapshot,
    get_project_context_snapshot, get_project_detail, get_settings_snapshot, get_tool_diagnostics,
    get_tools_snapshot, get_workspace_snapshot, import_github_repo_skills, import_local_skills,
    import_project_from_git, import_provider_config, import_rule_asset, install_skill_asset,
    list_providers, mark_skill_asset_stale, move_rule_asset, pick_file_path, pick_folder_path,
    preview_backup_restore, preview_first_run_import, preview_github_repo_import,
    preview_global_output, preview_local_skill_import, preview_project_output,
    preview_provider_apply, preview_repository_import, preview_rule_impact, preview_rule_import,
    repair_global_output, repair_project_output, repair_skill_asset, repair_tool, reset_app_data,
    reset_first_run_import_status, reset_project_output,
    restore_backup, save_project_entity, save_project_rule_bindings, save_provider,
    save_rule_asset, save_skill_asset, save_tool_credential_state, save_tool_global_rule_bindings,
    save_tool_skill_bindings, scan_library_diagnostics, scan_project_output, set_tool_enabled,
    uninstall_skill_asset, verify_tool_credential,
};
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            app.manage(AppContainer::new()?);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_app_bootstrap,
            reset_app_data,
            get_first_run_import_status,
            preview_first_run_import,
            apply_first_run_import,
            dismiss_first_run_import,
            reset_first_run_import_status,
            get_workspace_snapshot,
            get_project_context_snapshot,
            get_catalog_snapshot,
            get_library_snapshot,
            get_tools_snapshot,
            get_tool_diagnostics,
            set_tool_enabled,
            verify_tool_credential,
            repair_tool,
            pick_folder_path,
            pick_file_path,
            preview_repository_import,
            apply_repository_import,
            preview_github_repo_import,
            import_github_repo_skills,
            preview_local_skill_import,
            import_local_skills,
            import_project_from_git,
            save_project_entity,
            delete_project_entity,
            save_project_rule_bindings,
            save_tool_global_rule_bindings,
            save_tool_skill_bindings,
            save_rule_asset,
            import_rule_asset,
            preview_rule_import,
            move_rule_asset,
            preview_rule_impact,
            delete_rule_asset,
            save_skill_asset,
            delete_skill_asset,
            install_skill_asset,
            uninstall_skill_asset,
            repair_skill_asset,
            mark_skill_asset_stale,
            save_tool_credential_state,
            clear_tool_credential_state,
            scan_library_diagnostics,
            scan_project_output,
            preview_project_output,
            apply_project_output,
            repair_project_output,
            cleanup_project_output,
            reset_project_output,
            preview_global_output,
            apply_global_output,
            repair_global_output,
            cleanup_global_output,
            get_project_detail,
            get_history_snapshot,
            get_backup_snapshot,
            preview_backup_restore,
            restore_backup,
            delete_backup,
            export_library_diagnostics,
            get_settings_snapshot,
            list_providers,
            save_provider,
            import_provider_config,
            delete_provider,
            duplicate_provider,
            preview_provider_apply,
            apply_provider_to_live_config,
            detect_provider_live_drift
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
