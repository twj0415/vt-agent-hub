use crate::infrastructure::resource_repo::ResourceRepo;

use super::WriteService;

impl WriteService {
    #[cfg(test)]
    pub fn db_count(&self, table: &str) -> Result<i32, String> {
        let db = self.db.lock().expect("db poisoned");
        let sql = format!("select count(*) from {table}");
        db.connection()
            .query_row(&sql, [], |row| row.get(0))
            .map_err(|error| error.to_string())
    }

    #[cfg(test)]
    pub fn project_rule_ids(&self, project_id: i32, tool_id: i32) -> Result<Vec<i32>, String> {
        let db = self.db.lock().expect("db poisoned");
        let repo = ResourceRepo::new(&db);
        let mut ids = Vec::new();
        for binding in repo.project_rule_bindings(project_id)? {
            if binding.tool_id.is_some() && binding.tool_id != Some(tool_id) {
                continue;
            }
            for item in binding.items {
                if item.item_type == "rule" {
                    if !ids.contains(&item.asset_id) {
                        ids.push(item.asset_id);
                    }
                }
            }
        }
        Ok(ids)
    }
}
