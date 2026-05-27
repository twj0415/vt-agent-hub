use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::application::service_context::ServiceContext;
use crate::infrastructure::database::Database;

pub struct AppContainer {
    db: Arc<Mutex<Database>>,
    context: ServiceContext,
}

impl AppContainer {
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

    pub fn context(&self) -> &ServiceContext {
        &self.context
    }

    pub fn db(&self) -> Arc<Mutex<Database>> {
        self.db.clone()
    }

    pub fn release_database_for_reset(&self) -> Result<PathBuf, String> {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or(0);
        let temp_root = std::env::temp_dir().join(format!("vt-agent-hub-reset-{suffix}"));
        let temp_db = Database::open_at(&temp_root.join("app.db"))?;
        *self.db.lock().expect("db poisoned") = temp_db;
        Ok(temp_root)
    }

    pub fn replace_database(&self) -> Result<(), String> {
        let new_db = Database::open_at(self.context.db_path())?;
        *self.db.lock().expect("db poisoned") = new_db;
        Ok(())
    }
}
