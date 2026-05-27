use crate::application::operation_service::OperationService;
use crate::core::validation;
use crate::infrastructure::credential_store::CredentialStore;
use crate::infrastructure::settings_repo::SettingsRepo;

use super::WriteService;

impl WriteService {
    pub fn save_tool_credential(&self, tool_id: i32, token: &str) -> Result<(), String> {
        validation::validate_credential(tool_id, token)?;
        CredentialStore::save_tool_token(tool_id, token)?;

        let db = self.db.lock().expect("db poisoned");
        let key = format!("tool_{}_credential_state", tool_id);
        db.connection()
            .execute(
                "insert or replace into settings (key, value) values (?1, ?2)",
                rusqlite::params![key, "present"],
            )
            .map_err(|error| error.to_string())?;
        OperationService::record(
            &db,
            None,
            "operation",
            "Credential save",
            "credential-save",
            "Saved tool credential presence into backend state.",
        )?;
        Ok(())
    }

    pub fn clear_tool_credential(&self, tool_id: i32) -> Result<(), String> {
        CredentialStore::clear_tool_token(tool_id)?;

        let db = self.db.lock().expect("db poisoned");
        let key = format!("tool_{}_credential_state", tool_id);
        db.connection()
            .execute(
                "delete from settings where key = ?1",
                rusqlite::params![key],
            )
            .map_err(|error| error.to_string())?;
        OperationService::record(
            &db,
            None,
            "operation",
            "Credential clear",
            "credential-clear",
            "Cleared tool credential from backend state.",
        )?;
        Ok(())
    }

    pub fn has_tool_credential(&self, tool_id: i32) -> Result<bool, String> {
        let db = self.db.lock().expect("db poisoned");
        let key = format!("tool_{}_credential_state", tool_id);
        let repo = SettingsRepo::new(&db);
        let found = repo
            .list()?
            .into_iter()
            .any(|item| item.key == key && item.value == "present");
        Ok(found)
    }
}
