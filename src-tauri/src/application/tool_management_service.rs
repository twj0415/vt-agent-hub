use std::sync::{Arc, Mutex};

use crate::application::app_container::AppContainer;
use crate::application::operation_service::OperationService;
use crate::application::service_context::ServiceContext;
use crate::core::routes::ROUTE_SETTINGS;
use crate::core::status_codes::HEALTH_NORMAL;
use crate::infrastructure::database::Database;
use crate::infrastructure::tool_repo::ToolRepo;

pub struct ToolManagementService {
    db: Arc<Mutex<Database>>,
}

impl ToolManagementService {
    pub fn new() -> Result<Self, String> {
        Self::with_context(ServiceContext::default()?)
    }

    pub fn with_context(context: ServiceContext) -> Result<Self, String> {
        let db = context.open_db()?;
        Ok(Self {
            db: Arc::new(Mutex::new(db)),
        })
    }

    pub fn with_container(container: &AppContainer) -> Self {
        Self { db: container.db() }
    }

    pub fn set_enabled(&self, tool_id: i32, enabled: bool) -> Result<(), String> {
        let db = self.db.lock().expect("db poisoned");
        ToolRepo::new(&db).set_enabled(tool_id, enabled)?;
        OperationService::record_simple(
            &db,
            None,
            Some(tool_id),
            None,
            "operation",
            if enabled {
                "Tool enabled"
            } else {
                "Tool disabled"
            },
            "tool-set-enabled",
            if enabled {
                "Enabled tool."
            } else {
                "Disabled tool and cleaned related bindings and provider config."
            },
            "success",
            HEALTH_NORMAL,
            None,
            ROUTE_SETTINGS,
        )
    }
}
