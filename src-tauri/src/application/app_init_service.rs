use crate::application::app_container::AppContainer;
use crate::application::service_context::ServiceContext;
use crate::core::product::PRODUCT_NAME;
use crate::core::tool_registry::{get_tool, CLAUDE_TOOL_ID, CODEX_TOOL_ID};
use crate::domain::app_bootstrap::AppBootstrap;
use crate::infrastructure::credential_store::CredentialStore;
use crate::infrastructure::database::Database;

#[derive(Debug)]
pub struct AppInitService {
    context: ServiceContext,
}

impl AppInitService {
    pub fn new() -> Result<Self, String> {
        Self::with_context(ServiceContext::default()?)
    }

    pub fn with_context(context: ServiceContext) -> Result<Self, String> {
        Ok(Self { context })
    }

    pub fn with_container(container: &AppContainer) -> Self {
        Self {
            context: container.context().clone(),
        }
    }

    pub fn get_bootstrap(&self) -> Result<AppBootstrap, String> {
        let _db = self.context.open_db()?;
        let active_tool = get_tool(CODEX_TOOL_ID)
            .map(|tool| tool.id)
            .or_else(|| get_tool(CLAUDE_TOOL_ID).map(|tool| tool.id))
            .unwrap_or(CODEX_TOOL_ID);

        Ok(AppBootstrap {
            app_name: PRODUCT_NAME.to_string(),
            active_tool_id: active_tool,
        })
    }

    pub fn reset_app_data(&self) -> Result<String, String> {
        Self::reset_context_data(&self.context)?;
        let _db = Database::open_at(self.context.db_path())?;
        Ok(Self::reset_message())
    }

    pub fn reset_app_data_with_container(container: &AppContainer) -> Result<String, String> {
        let temp_root = container.release_database_for_reset()?;
        let result = (|| {
            Self::reset_context_data(container.context())?;
            container.replace_database()?;
            Ok(Self::reset_message())
        })();
        let _ = std::fs::remove_dir_all(temp_root);
        result
    }

    fn reset_context_data(context: &ServiceContext) -> Result<(), String> {
        let storage_root = context.storage_root()?;
        CredentialStore::clear_tool_token(CODEX_TOOL_ID)?;
        if storage_root.exists() {
            let canonical_storage = storage_root
                .canonicalize()
                .map_err(|error| format!("Failed to resolve storage root: {error}"))?;

            for entry in std::fs::read_dir(&canonical_storage).map_err(|error| error.to_string())? {
                let entry = entry.map_err(|error| error.to_string())?;
                let path = entry.path();
                let canonical_path = path
                    .canonicalize()
                    .map_err(|error| format!("Failed to resolve reset target: {error}"))?;
                if !canonical_path.starts_with(&canonical_storage) {
                    return Err(format!(
                        "Refusing to reset path outside storage root: {}",
                        canonical_path.display()
                    ));
                }
                if canonical_path.is_dir() {
                    std::fs::remove_dir_all(&canonical_path).map_err(|error| error.to_string())?;
                } else {
                    std::fs::remove_file(&canonical_path).map_err(|error| error.to_string())?;
                }
            }
        }
        Ok(())
    }

    fn reset_message() -> String {
        format!(
            "{} data was reset to an empty initialized state.",
            PRODUCT_NAME
        )
    }
}
