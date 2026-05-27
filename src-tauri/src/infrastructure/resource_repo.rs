mod mutations;
mod project_bindings;
mod queries;
mod records;
mod skill_installs;
mod tool_bindings;

use crate::infrastructure::database::Database;

pub use records::*;

pub struct ResourceRepo<'a> {
    db: &'a Database,
}

impl<'a> ResourceRepo<'a> {
    pub fn new(db: &'a Database) -> Self {
        Self { db }
    }
}
