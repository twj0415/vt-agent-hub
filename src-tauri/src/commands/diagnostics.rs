use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::application::app_container::AppContainer;
use crate::application::backup_service::BackupService;
use crate::application::library_diagnostics_service::LibraryDiagnosticsService;
use crate::commands::history_log::{record_command_failure, CommandFailure};
use crate::dto::{
    AppResponse, BackupActionResultDto, BackupRestorePreviewDto, BackupSnapshotDto,
    DiagnosticExportResultDto, LibraryDiagnosticsDto,
};
use crate::infrastructure::database::Database;

#[tauri::command]
pub fn scan_library_diagnostics(
    state: tauri::State<'_, AppContainer>,
) -> AppResponse<LibraryDiagnosticsDto> {
    let service = match LibraryDiagnosticsService::with_container(state.inner()) {
        Ok(service) => service,
        Err(error) => {
            return AppResponse::error(
                "library_diagnostics_failed",
                &error,
                "errors.libraryDiagnosticsFailed",
            )
        }
    };

    match service.scan() {
        Ok(result) => AppResponse::success(result),
        Err(error) => AppResponse::error(
            "library_diagnostics_failed",
            &error,
            "errors.libraryDiagnosticsFailed",
        ),
    }
}

#[tauri::command]
pub fn get_backup_snapshot(
    state: tauri::State<'_, AppContainer>,
) -> AppResponse<BackupSnapshotDto> {
    let service = match BackupService::with_container(state.inner()) {
        Ok(service) => service,
        Err(error) => {
            return AppResponse::error(
                "backup_snapshot_failed",
                &error,
                "errors.backupSnapshotFailed",
            )
        }
    };

    match service.list() {
        Ok(result) => AppResponse::success(result),
        Err(error) => AppResponse::error(
            "backup_snapshot_failed",
            &error,
            "errors.backupSnapshotFailed",
        ),
    }
}

#[tauri::command]
pub fn preview_backup_restore(
    state: tauri::State<'_, AppContainer>,
    backup_id: String,
) -> AppResponse<BackupRestorePreviewDto> {
    let service = match BackupService::with_container(state.inner()) {
        Ok(service) => service,
        Err(error) => {
            return AppResponse::error(
                "backup_restore_preview_failed",
                &error,
                "errors.backupRestorePreviewFailed",
            )
        }
    };

    match service.preview_restore(&backup_id) {
        Ok(result) => AppResponse::success(result),
        Err(error) => {
            record_history_failure(
                "repair",
                "Backup restore preview failed",
                "backup-preview-restore",
                &error,
                Some(&backup_id),
                "/history",
            );
            AppResponse::error(
                "backup_restore_preview_failed",
                &error,
                "errors.backupRestorePreviewFailed",
            )
        }
    }
}

#[tauri::command]
pub fn restore_backup(
    state: tauri::State<'_, AppContainer>,
    backup_id: String,
    confirm_risk: bool,
) -> AppResponse<BackupActionResultDto> {
    let service = match BackupService::with_container(state.inner()) {
        Ok(service) => service,
        Err(error) => {
            return AppResponse::error(
                "backup_restore_failed",
                &error,
                "errors.backupRestoreFailed",
            )
        }
    };

    match service.restore(&backup_id, confirm_risk) {
        Ok(result) => AppResponse::success(result),
        Err(error) => {
            record_history_failure(
                "repair",
                "Backup restore failed",
                "backup-restore",
                &error,
                Some(&backup_id),
                "/history",
            );
            AppResponse::error(
                "backup_restore_failed",
                &error,
                "errors.backupRestoreFailed",
            )
        }
    }
}

#[tauri::command]
pub fn delete_backup(
    state: tauri::State<'_, AppContainer>,
    backup_id: String,
) -> AppResponse<BackupActionResultDto> {
    let service = match BackupService::with_container(state.inner()) {
        Ok(service) => service,
        Err(error) => {
            return AppResponse::error("backup_delete_failed", &error, "errors.backupDeleteFailed")
        }
    };

    match service.delete(&backup_id) {
        Ok(result) => AppResponse::success(result),
        Err(error) => {
            record_history_failure(
                "operation",
                "Backup delete failed",
                "backup-delete",
                &error,
                Some(&backup_id),
                "/history",
            );
            AppResponse::error("backup_delete_failed", &error, "errors.backupDeleteFailed")
        }
    }
}

#[tauri::command]
pub fn export_library_diagnostics(
    state: tauri::State<'_, AppContainer>,
) -> AppResponse<DiagnosticExportResultDto> {
    let service = match LibraryDiagnosticsService::with_container(state.inner()) {
        Ok(service) => service,
        Err(error) => {
            return AppResponse::error(
                "diagnostics_export_failed",
                &error,
                "errors.diagnosticsExportFailed",
            )
        }
    };

    match service.scan().and_then(|diagnostics| {
        let root = Database::snapshots_root()?;
        fs::create_dir_all(&root).map_err(|error| error.to_string())?;
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_secs())
            .unwrap_or(0);
        let path = root.join(format!("library-diagnostics-{timestamp}.json"));
        let text = serde_json::to_string_pretty(&diagnostics).map_err(|error| error.to_string())?;
        fs::write(&path, text).map_err(|error| error.to_string())?;
        Ok(DiagnosticExportResultDto {
            path: path.display().to_string(),
            issue_count: diagnostics.issue_count,
            message: "Diagnostics exported.".to_string(),
        })
    }) {
        Ok(result) => AppResponse::success(result),
        Err(error) => {
            record_history_failure(
                "diagnostic",
                "Diagnostics export failed",
                "library-diagnostics-export",
                &error,
                None,
                "/history",
            );
            AppResponse::error(
                "diagnostics_export_failed",
                &error,
                "errors.diagnosticsExportFailed",
            )
        }
    }
}

fn record_history_failure(
    kind: &str,
    title: &str,
    action: &str,
    detail: &str,
    related_path: Option<&str>,
    navigation_target: &str,
) {
    record_command_failure(CommandFailure {
        project_id: None,
        tool_id: None,
        related_rule_id: None,
        kind,
        title,
        action,
        detail,
        related_path,
        navigation_target,
    });
}
