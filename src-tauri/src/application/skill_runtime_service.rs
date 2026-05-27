use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use crate::application::app_container::AppContainer;
use crate::application::operation_service::OperationService;
use crate::application::service_context::ServiceContext;
use crate::application::tool_service::ToolService;
use crate::core::routes::ROUTE_SKILLS;
use crate::core::status_codes::{
    HEALTH_NORMAL, HEALTH_WARNING, SKILL_INSTALL_CONFLICT, SKILL_INSTALL_ERROR,
    SKILL_INSTALL_INSTALLED, SKILL_INSTALL_NOT_INSTALLED, SKILL_INSTALL_SOURCE_MISSING,
    SKILL_INSTALL_STALE,
};
use crate::core::tool_registry::CODEX_TOOL_ID;
use crate::dto::{SkillFileNodeDto, SkillRuntimeDto};
use crate::infrastructure::database::Database;
use crate::infrastructure::resource_repo::ResourceRepo;

pub struct SkillRuntimeService {
    db: Arc<Mutex<Database>>,
    context: ServiceContext,
    tool_service: ToolService,
}

impl SkillRuntimeService {
    pub fn new() -> Result<Self, String> {
        Self::with_context(ServiceContext::default()?)
    }

    pub fn with_context(context: ServiceContext) -> Result<Self, String> {
        let db = context.open_db()?;
        Ok(Self {
            db: Arc::new(Mutex::new(db)),
            tool_service: ToolService::new(),
            context,
        })
    }

    pub fn with_container(container: &AppContainer) -> Self {
        Self {
            db: container.db(),
            tool_service: ToolService::new(),
            context: container.context().clone(),
        }
    }

    pub fn with_db_arc(db: Arc<Mutex<Database>>, context: ServiceContext) -> Self {
        Self {
            db,
            tool_service: ToolService::new(),
            context,
        }
    }

    pub fn inspect_skill(&self, skill_id: i32) -> Result<SkillRuntimeDto, String> {
        self.inspect_skill_for_tool(CODEX_TOOL_ID, skill_id)
    }

    pub fn inspect_skill_for_tool(
        &self,
        tool_id: i32,
        skill_id: i32,
    ) -> Result<SkillRuntimeDto, String> {
        let skill = {
            let db = self.db.lock().expect("db poisoned");
            ResourceRepo::new(&db).find_latest_skill_version_by_asset(skill_id)?
        };
        self.inspect_skill_by_name_for_tool(tool_id, &skill.name, &skill.body)
    }

    pub fn install_skill(&self, skill_id: i32) -> Result<SkillRuntimeDto, String> {
        self.install_skill_for_tool(CODEX_TOOL_ID, skill_id)
    }

    pub fn install_skill_for_tool(
        &self,
        tool_id: i32,
        skill_id: i32,
    ) -> Result<SkillRuntimeDto, String> {
        let skill = {
            let db = self.db.lock().expect("db poisoned");
            ResourceRepo::new(&db).find_latest_skill_version_by_asset(skill_id)?
        };
        let inspected = self.inspect_skill_by_name_for_tool(tool_id, &skill.name, &skill.body)?;

        if !inspected.library_exists {
            let message = "Skill source directory is missing in library/skills.";
            self.record_failure(
                skill_id,
                tool_id,
                "Skill install failed",
                "skill-install",
                message,
                Some(&inspected.library_path),
            )?;
            return Err(message.to_string());
        }

        if !inspected.skill_md_valid {
            let message = "Skill source SKILL.md is missing or empty.";
            self.record_failure(
                skill_id,
                tool_id,
                "Skill install failed",
                "skill-install",
                message,
                Some(&inspected.library_skill_md_path),
            )?;
            return Err(message.to_string());
        }

        if inspected.install_state == SKILL_INSTALL_CONFLICT {
            let message = "Runtime copy differs from the central asset. Use repair to overwrite it after review.";
            self.record_failure(
                skill_id,
                tool_id,
                "Skill install failed",
                "skill-install",
                message,
                Some(&inspected.runtime_path),
            )?;
            return Err(message.to_string());
        }

        if let Some(parent) = Path::new(&inspected.runtime_path).parent() {
            fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }

        if Path::new(&inspected.runtime_path).exists() {
            fs::remove_dir_all(&inspected.runtime_path).map_err(|error| error.to_string())?;
        }

        copy_dir_all(
            Path::new(&inspected.library_path),
            Path::new(&inspected.runtime_path),
        )?;
        let refreshed = self.inspect_skill_for_tool(tool_id, skill_id)?;
        self.record_success(
            skill_id,
            tool_id,
            "Skill install",
            "skill-install",
            &format!("Installed skill '{}' into tool runtime.", skill.name),
            Some(&refreshed.runtime_path),
        )?;
        Ok(refreshed)
    }

    pub fn uninstall_skill(&self, skill_id: i32) -> Result<SkillRuntimeDto, String> {
        self.uninstall_skill_for_tool(CODEX_TOOL_ID, skill_id)
    }

    pub fn uninstall_skill_for_tool(
        &self,
        tool_id: i32,
        skill_id: i32,
    ) -> Result<SkillRuntimeDto, String> {
        let skill = {
            let db = self.db.lock().expect("db poisoned");
            ResourceRepo::new(&db).find_latest_skill_version_by_asset(skill_id)?
        };
        let inspected = self.inspect_skill_by_name_for_tool(tool_id, &skill.name, &skill.body)?;

        if Path::new(&inspected.runtime_path).exists() {
            fs::remove_dir_all(&inspected.runtime_path).map_err(|error| error.to_string())?;
        }

        let refreshed = self.inspect_skill_for_tool(tool_id, skill_id)?;
        self.record_success(
            skill_id,
            tool_id,
            "Skill uninstall",
            "skill-uninstall",
            &format!("Removed skill '{}' from Codex runtime.", skill.name),
            Some(&inspected.runtime_path),
        )?;
        Ok(refreshed)
    }

    pub fn repair_skill(&self, skill_id: i32) -> Result<SkillRuntimeDto, String> {
        self.repair_skill_for_tool(CODEX_TOOL_ID, skill_id)
    }

    pub fn repair_skill_for_tool(
        &self,
        tool_id: i32,
        skill_id: i32,
    ) -> Result<SkillRuntimeDto, String> {
        let inspected = self.inspect_skill_for_tool(tool_id, skill_id)?;
        if !inspected.library_exists {
            let message = "Skill source directory is missing in library/skills.";
            self.record_failure(
                skill_id,
                tool_id,
                "Skill repair failed",
                "skill-repair",
                message,
                Some(&inspected.library_path),
            )?;
            return Err(message.to_string());
        }

        let repaired = self.install_skill_for_tool(tool_id, skill_id)?;
        self.record_success(
            skill_id,
            tool_id,
            "Skill repair",
            "skill-repair",
            "Reinstalled skill from central library asset into Codex runtime.",
            Some(&repaired.runtime_path),
        )?;
        Ok(repaired)
    }

    pub fn mark_skill_stale(&self, skill_id: i32) -> Result<SkillRuntimeDto, String> {
        self.mark_skill_stale_for_tool(CODEX_TOOL_ID, skill_id)
    }

    pub fn mark_skill_stale_for_tool(
        &self,
        tool_id: i32,
        skill_id: i32,
    ) -> Result<SkillRuntimeDto, String> {
        let skill = {
            let db = self.db.lock().expect("db poisoned");
            ResourceRepo::new(&db).find_latest_skill_version_by_asset(skill_id)?
        };
        let inspected = self.inspect_skill_by_name_for_tool(tool_id, &skill.name, &skill.body)?;
        let runtime_skill_md_path = PathBuf::from(&inspected.runtime_skill_md_path);

        if !runtime_skill_md_path.is_file() {
            let message = "Runtime SKILL.md is missing, so stale marking cannot be applied.";
            self.record_failure(
                skill_id,
                tool_id,
                "Skill mark stale failed",
                "skill-mark-stale",
                message,
                Some(&inspected.runtime_path),
            )?;
            return Err(message.to_string());
        }

        let current =
            fs::read_to_string(&runtime_skill_md_path).map_err(|error| error.to_string())?;
        let stale_body = if current.contains("\n\n<!-- stale marker -->\n") {
            current
        } else {
            format!("{current}\n\n<!-- stale marker -->\n")
        };
        fs::write(&runtime_skill_md_path, stale_body).map_err(|error| error.to_string())?;

        let refreshed = self.inspect_skill_for_tool(tool_id, skill_id)?;
        self.record_success(
            skill_id,
            tool_id,
            "Skill mark stale",
            "skill-mark-stale",
            "Marked runtime copy stale to force repair comparison.",
            Some(&refreshed.runtime_skill_md_path),
        )?;
        Ok(refreshed)
    }

    pub fn inspect_skill_by_name(
        &self,
        name: &str,
        library_body_fallback: &str,
    ) -> SkillRuntimeDto {
        self.inspect_skill_by_name_for_tool(CODEX_TOOL_ID, name, library_body_fallback)
            .expect("Codex skill runtime root should be available")
    }

    pub fn inspect_skill_by_name_for_tool(
        &self,
        tool_id: i32,
        name: &str,
        library_body_fallback: &str,
    ) -> Result<SkillRuntimeDto, String> {
        let library_root = self
            .context
            .library_root()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join("skills")
            .join(name);
        let runtime_platform_root = self.tool_service.skill_runtime_root(tool_id)?;
        let runtime_root = runtime_platform_root.join(name);
        let library_skill_md = library_root.join("SKILL.md");
        let runtime_skill_md = runtime_root.join("SKILL.md");
        let library_exists = library_root.is_dir();
        let runtime_exists = runtime_root.is_dir();
        let library_body =
            read_if_file(&library_skill_md).unwrap_or_else(|| library_body_fallback.to_string());
        let runtime_body = read_if_file(&runtime_skill_md).unwrap_or_default();
        let skill_md_valid = !library_body.trim().is_empty() && library_skill_md.is_file();
        let install_state = determine_install_state(
            library_exists,
            runtime_exists,
            skill_md_valid,
            &library_body,
            &runtime_body,
        );
        let status_detail = status_detail_message(
            install_state,
            library_exists,
            runtime_exists,
            skill_md_valid,
            &library_root,
            &runtime_root,
        );
        let install_action_ready = library_exists
            && skill_md_valid
            && matches!(
                install_state,
                SKILL_INSTALL_NOT_INSTALLED | SKILL_INSTALL_STALE
            );
        let uninstall_action_ready = runtime_exists;
        let repair_action_ready = library_exists
            && skill_md_valid
            && matches!(
                install_state,
                SKILL_INSTALL_STALE | SKILL_INSTALL_CONFLICT | SKILL_INSTALL_ERROR
            );
        let mark_stale_action_ready = runtime_exists && install_state == SKILL_INSTALL_INSTALLED;

        Ok(SkillRuntimeDto {
            platform_root: runtime_platform_root.display().to_string(),
            library_path: library_root.display().to_string(),
            library_skill_md_path: library_skill_md.display().to_string(),
            runtime_path: runtime_root.display().to_string(),
            runtime_skill_md_path: runtime_skill_md.display().to_string(),
            library_exists,
            runtime_exists,
            skill_md_valid,
            install_state,
            status_detail,
            library_body,
            runtime_body,
            library_tree: list_tree(&library_root, ""),
            runtime_tree: list_tree(&runtime_root, ""),
            install_action_ready,
            uninstall_action_ready,
            repair_action_ready,
            mark_stale_action_ready,
        })
    }

    fn record_success(
        &self,
        skill_id: i32,
        tool_id: i32,
        title: &str,
        action: &str,
        detail: &str,
        related_path: Option<&str>,
    ) -> Result<(), String> {
        let db = self.db.lock().expect("db poisoned");
        OperationService::record_simple(
            &db,
            None,
            Some(tool_id),
            Some(skill_id),
            "operation",
            title,
            action,
            detail,
            "success",
            HEALTH_NORMAL,
            related_path,
            ROUTE_SKILLS,
        )
    }

    fn record_failure(
        &self,
        skill_id: i32,
        tool_id: i32,
        title: &str,
        action: &str,
        detail: &str,
        related_path: Option<&str>,
    ) -> Result<(), String> {
        let db = self.db.lock().expect("db poisoned");
        OperationService::record_simple(
            &db,
            None,
            Some(tool_id),
            Some(skill_id),
            "operation",
            title,
            action,
            detail,
            "failed",
            HEALTH_WARNING,
            related_path,
            ROUTE_SKILLS,
        )
    }
}

fn determine_install_state(
    library_exists: bool,
    runtime_exists: bool,
    skill_md_valid: bool,
    library_body: &str,
    runtime_body: &str,
) -> i32 {
    if !library_exists {
        return SKILL_INSTALL_SOURCE_MISSING;
    }
    if !skill_md_valid {
        return SKILL_INSTALL_ERROR;
    }
    if !runtime_exists {
        return SKILL_INSTALL_NOT_INSTALLED;
    }
    if runtime_body.trim().is_empty() {
        return SKILL_INSTALL_ERROR;
    }
    if runtime_body == library_body {
        return SKILL_INSTALL_INSTALLED;
    }
    if runtime_body.contains("<!-- stale marker -->") {
        return SKILL_INSTALL_STALE;
    }
    SKILL_INSTALL_CONFLICT
}

fn status_detail_message(
    install_state: i32,
    library_exists: bool,
    runtime_exists: bool,
    skill_md_valid: bool,
    library_root: &Path,
    runtime_root: &Path,
) -> String {
    match install_state {
        SKILL_INSTALL_SOURCE_MISSING => format!(
            "Central asset is missing at {}.",
            library_root.display()
        ),
        SKILL_INSTALL_NOT_INSTALLED => format!(
            "Central asset exists, but no runtime copy is installed at {}.",
            runtime_root.display()
        ),
        SKILL_INSTALL_INSTALLED => "Runtime copy matches the central skill asset.".to_string(),
        SKILL_INSTALL_STALE => "Runtime copy is marked stale and should be repaired from the central asset.".to_string(),
        SKILL_INSTALL_CONFLICT => "Runtime copy differs from the central asset. Review or repair before trusting install state.".to_string(),
        SKILL_INSTALL_ERROR if !skill_md_valid => "Central SKILL.md is missing or empty.".to_string(),
        SKILL_INSTALL_ERROR if !runtime_exists => "Runtime install path is incomplete.".to_string(),
        SKILL_INSTALL_ERROR if !library_exists => "Central skill asset cannot be read.".to_string(),
        _ => "Skill runtime status could not be resolved cleanly.".to_string(),
    }
}

fn read_if_file(path: &Path) -> Option<String> {
    if path.is_file() {
        fs::read_to_string(path).ok()
    } else {
        None
    }
}

fn list_tree(root: &Path, prefix: &str) -> Vec<SkillFileNodeDto> {
    if !root.exists() {
        return Vec::new();
    }

    let mut entries = match fs::read_dir(root) {
        Ok(rows) => rows.filter_map(Result::ok).collect::<Vec<_>>(),
        Err(_) => return Vec::new(),
    };
    entries.sort_by_key(|entry| entry.file_name());

    let mut result = Vec::new();
    for entry in entries {
        let name = entry.file_name().to_string_lossy().to_string();
        let relative = if prefix.is_empty() {
            name.clone()
        } else {
            format!("{prefix}/{name}")
        };
        let is_dir = entry.path().is_dir();
        result.push(SkillFileNodeDto {
            path: relative.clone(),
            is_dir,
        });
        if is_dir {
            result.extend(list_tree(&entry.path(), &relative));
        }
    }
    result
}

fn copy_dir_all(src: &Path, dst: &Path) -> Result<(), String> {
    fs::create_dir_all(dst).map_err(|error| error.to_string())?;
    for entry in fs::read_dir(src).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
        let path = entry.path();
        let target = dst.join(entry.file_name());
        if path.is_dir() {
            copy_dir_all(&path, &target)?;
        } else {
            fs::copy(&path, &target).map_err(|error| error.to_string())?;
        }
    }
    Ok(())
}
