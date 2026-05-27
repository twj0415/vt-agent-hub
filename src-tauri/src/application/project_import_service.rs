use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};

use crate::application::app_container::AppContainer;
use crate::application::operation_service::OperationService;
use crate::application::service_context::ServiceContext;
use crate::application::write_service::WriteService;
use crate::dto::ProjectDetailDto;
use crate::infrastructure::database::Database;

pub struct ProjectImportService {
    db: Arc<Mutex<Database>>,
    context: ServiceContext,
}

impl ProjectImportService {
    pub fn new() -> Result<Self, String> {
        Self::with_context(ServiceContext::default()?)
    }

    pub fn with_context(context: ServiceContext) -> Result<Self, String> {
        let db = context.open_db()?;
        Ok(Self {
            db: Arc::new(Mutex::new(db)),
            context,
        })
    }

    pub fn with_container(container: &AppContainer) -> Self {
        Self {
            db: container.db(),
            context: container.context().clone(),
        }
    }

    pub fn import_from_git(
        &self,
        repo_url: &str,
        target_path: &str,
        project_name: Option<&str>,
        branch: Option<&str>,
        project_type: i32,
    ) -> Result<ProjectDetailDto, String> {
        let repo_url = repo_url.trim();
        let target_path = target_path.trim();
        if repo_url.is_empty() {
            return Err("Repository URL is required.".to_string());
        }
        if target_path.is_empty() {
            return Err("Target path is required.".to_string());
        }

        let target_dir = PathBuf::from(target_path);
        if target_dir.exists() {
            return Err("Target path already exists.".to_string());
        }

        let checkout_dir = self.clone_repository(repo_url, &target_dir, branch)?;
        let name = project_name
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string)
            .unwrap_or_else(|| {
                checkout_dir
                    .file_name()
                    .and_then(|value| value.to_str())
                    .unwrap_or(target_path)
                    .to_string()
            });
        // 共享 self.db,保证 save_project 内部写与本服务后续审计写在同一连接上。
        let write_service = WriteService::with_db_arc(self.db.clone(), self.context.clone());
        let saved_id = match write_service.save_project(
            None,
            &name,
            checkout_dir.to_string_lossy().as_ref(),
            project_type,
            true,
        ) {
            Ok(saved_id) => saved_id,
            Err(error) => {
                let _ = fs::remove_dir_all(&checkout_dir);
                return Err(error);
            }
        };
        let db = self.db.lock().expect("db poisoned");
        OperationService::record(
            &db,
            Some(saved_id),
            "operation",
            "Project import",
            "project-import-git",
            &format!(
                "Imported project from git repository into {}.",
                checkout_dir.display()
            ),
        )?;

        Ok(ProjectDetailDto {
            id: saved_id,
            name,
            path: checkout_dir.to_string_lossy().to_string(),
            project_type,
            rule_bindings: Vec::new(),
            last_operation: "Project imported.".to_string(),
            latest_backup: String::new(),
        })
    }

    fn clone_repository(
        &self,
        repo_url: &str,
        target_dir: &Path,
        branch: Option<&str>,
    ) -> Result<PathBuf, String> {
        if let Some(parent) = target_dir
            .parent()
            .filter(|path| !path.as_os_str().is_empty())
        {
            fs::create_dir_all(parent)
                .map_err(|error| format!("Failed to create target parent directory: {error}"))?;
        }

        let mut cmd = Command::new("git");
        cmd.arg("clone");
        if let Some(branch) = branch.filter(|value| !value.trim().is_empty()) {
            cmd.args(["--branch", branch]);
        }
        cmd.arg(repo_url).arg(target_dir);

        let output = cmd
            .output()
            .map_err(|error| format!("Failed to start git clone: {error}"))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("git clone failed: {stderr}"));
        }

        Ok(target_dir.to_path_buf())
    }
}
