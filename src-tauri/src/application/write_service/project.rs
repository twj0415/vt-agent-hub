use crate::application::operation_service::OperationService;
use crate::core::validation;
use crate::infrastructure::project_repo::ProjectRepo;

use super::WriteService;

impl WriteService {
    pub fn save_project(
        &self,
        id: Option<i32>,
        name: &str,
        path: &str,
        project_type: i32,
        import_mode: bool,
    ) -> Result<i32, String> {
        validation::validate_project(name, path, project_type, import_mode)?;
        let db = self.db.lock().expect("db poisoned");
        let repo = ProjectRepo::new(&db);
        let saved_id = repo.upsert(id, name, path, project_type)?;
        OperationService::record(
            &db,
            Some(saved_id),
            "operation",
            if import_mode {
                "Project import"
            } else {
                "Project save"
            },
            "project-write",
            if import_mode {
                "Imported project entity into SQLite."
            } else if id.is_some() {
                "Updated project entity in SQLite."
            } else {
                "Created project entity in SQLite."
            },
        )?;
        Ok(saved_id)
    }

    pub fn delete_project(&self, project_id: i32) -> Result<(), String> {
        let db = self.db.lock().expect("db poisoned");
        let repo = ProjectRepo::new(&db);
        repo.delete(project_id)?;
        OperationService::record(
            &db,
            Some(project_id),
            "operation",
            "Project delete",
            "project-delete",
            "Deleted project entity from SQLite.",
        )?;
        Ok(())
    }
}
