use crate::application::operation_service::OperationService;
use crate::core::validation;
use crate::infrastructure::resource_repo::ResourceRepo;

use super::WriteService;

impl WriteService {
    pub fn replace_project_rule_bindings(
        &self,
        project_id: i32,
        tool_id: Option<i32>,
        rule_ids: &[i32],
    ) -> Result<(), String> {
        validation::validate_project_rule_binding_input(project_id, tool_id, rule_ids)?;
        let db = self.db.lock().expect("db poisoned");
        let repo = ResourceRepo::new(&db);
        repo.replace_project_rule_binding_from_rules(project_id, tool_id, rule_ids)?;
        let detail = if rule_ids.is_empty() {
            match tool_id {
                Some(tool_id) => format!("Cleared project rule bindings for tool {}.", tool_id),
                None => "Cleared project common rule bindings.".to_string(),
            }
        } else {
            match tool_id {
                Some(tool_id) => format!(
                    "Updated {} project rule bindings for tool {}.",
                    rule_ids.len(),
                    tool_id
                ),
                None => format!("Updated {} project common rule bindings.", rule_ids.len()),
            }
        };
        OperationService::record(
            &db,
            Some(project_id),
            "operation",
            "Rule binding update",
            "project-rule-bindings",
            &detail,
        )?;
        Ok(())
    }

    pub fn replace_tool_global_rule_bindings(
        &self,
        tool_id: i32,
        rule_ids: &[i32],
    ) -> Result<(), String> {
        validation::validate_binding_input(tool_id, tool_id, rule_ids)?;
        let db = self.db.lock().expect("db poisoned");
        let repo = ResourceRepo::new(&db);
        repo.replace_tool_global_rule_binding_from_rules(tool_id, rule_ids)?;
        OperationService::record(
            &db,
            None,
            "operation",
            "Tool global rule binding update",
            "tool-global-rule-bindings",
            &format!("Updated {} tool-global rule bindings.", rule_ids.len()),
        )?;
        Ok(())
    }

    pub fn replace_tool_skill_bindings(
        &self,
        tool_id: i32,
        skill_ids: &[i32],
    ) -> Result<(), String> {
        validation::validate_tool_skill_binding_input(tool_id, skill_ids)?;
        let db = self.db.lock().expect("db poisoned");
        let repo = ResourceRepo::new(&db);
        repo.replace_tool_skill_binding_from_skills(tool_id, skill_ids)?;
        OperationService::record(
            &db,
            None,
            "operation",
            "Tool skill binding update",
            "tool-skill-bindings",
            &format!("Updated {} tool skill bindings.", skill_ids.len()),
        )?;
        Ok(())
    }
}
