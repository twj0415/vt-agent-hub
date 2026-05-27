mod binding;
mod credential;
mod markdown;
mod project;
mod rule;
mod skill;
#[cfg(test)]
mod test_helpers;

use std::sync::{Arc, Mutex};

use crate::application::app_container::AppContainer;
use crate::application::service_context::ServiceContext;
use crate::infrastructure::database::Database;

pub use markdown::{markdown_description, parse_markdown_rule, MarkdownRuleParts};

pub struct WriteService {
    db: Arc<Mutex<Database>>,
    context: ServiceContext,
}

impl WriteService {
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

    /// 让上层 service 在持有同一 Arc 时复用 WriteService。仅供同一 service 链复用。
    pub fn with_db_arc(db: Arc<Mutex<Database>>, context: ServiceContext) -> Self {
        Self { db, context }
    }

    pub(super) fn asset_key(name: &str) -> String {
        let lowered = name.trim().to_lowercase();
        let mut key = String::new();
        let mut last_dash = false;

        for ch in lowered.chars() {
            if ch.is_alphanumeric() {
                key.push(ch);
                last_dash = false;
            } else if !last_dash {
                key.push('-');
                last_dash = true;
            }
        }

        let key = key.trim_matches('-').to_string();
        if key.is_empty() {
            let hash = name.bytes().fold(0xcbf29ce484222325u64, |acc, byte| {
                (acc ^ u64::from(byte)).wrapping_mul(0x100000001b3)
            });
            format!("rule-{hash:016x}")
        } else {
            key
        }
    }
}
