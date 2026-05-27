use crate::application::operation_service::OperationService;
use crate::core::validation;
use crate::infrastructure::resource_repo::ResourceRepo;
use crate::infrastructure::skill_asset_repo::SkillAssetRepo;

use super::WriteService;

impl WriteService {
    pub fn save_skill(
        &self,
        id: Option<i32>,
        code: i32,
        name: &str,
        category_code: i32,
        state: i32,
        install_state: i32,
        summary: &str,
        body: &str,
    ) -> Result<i32, String> {
        validation::validate_skill(code, name, category_code, state, install_state, body)?;
        let db = self.db.lock().expect("db poisoned");
        let repo = ResourceRepo::new(&db);
        let previous_name = if let Some(existing_id) = id {
            Some(repo.find_latest_skill_version_by_asset(existing_id)?.name)
        } else {
            None
        };
        let saved = repo.save_skill_version(
            id,
            &Self::asset_key(name),
            code,
            name,
            category_code,
            state,
            summary,
            body,
        )?;
        let asset_repo = SkillAssetRepo::new(self.context.clone());
        if let Some(previous_name) = previous_name {
            asset_repo.rename_skill(&previous_name, name)?;
        }
        asset_repo.write_skill(name, body)?;
        OperationService::record(
            &db,
            None,
            "operation",
            if id.is_some() {
                "Skill update"
            } else {
                "Skill create"
            },
            "skill-write",
            if id.is_some() {
                "Updated skill asset in SQLite."
            } else {
                "Created skill asset in SQLite."
            },
        )?;
        Ok(saved.asset_id)
    }

    pub fn delete_skill(&self, id: i32) -> Result<(), String> {
        let db = self.db.lock().expect("db poisoned");
        let repo = ResourceRepo::new(&db);
        let skill = repo.find_latest_skill_version_by_asset(id)?;
        let bound_tool_count = repo.skill_bound_tool_count(id)?;
        if bound_tool_count > 0 {
            return Err(format!(
                "Skill is still bound to {} tool(s). Unbind it before deleting.",
                bound_tool_count
            ));
        }
        SkillAssetRepo::new(self.context.clone()).delete_skill(&skill.name)?;
        repo.delete_skill_asset(id)?;
        OperationService::record(
            &db,
            None,
            "operation",
            "Skill delete",
            "skill-delete",
            "Deleted skill asset from SQLite.",
        )?;
        Ok(())
    }
}
