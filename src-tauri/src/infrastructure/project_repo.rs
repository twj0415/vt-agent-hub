use rusqlite::{params, OptionalExtension};

use crate::infrastructure::database::Database;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectRecord {
    pub id: i32,
    pub name: String,
    pub path: String,
    pub project_type: i32,
}

pub struct ProjectRepo<'a> {
    db: &'a Database,
}

impl<'a> ProjectRepo<'a> {
    pub fn new(db: &'a Database) -> Self {
        Self { db }
    }

    pub fn list(&self) -> Result<Vec<ProjectRecord>, String> {
        let mut stmt = self
            .db
            .connection()
            .prepare("select id, name, path, project_type from projects order by id asc")
            .map_err(|error| error.to_string())?;

        let rows = stmt
            .query_map([], |row| {
                Ok(ProjectRecord {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    path: row.get(2)?,
                    project_type: row.get(3)?,
                })
            })
            .map_err(|error| error.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| error.to_string())?;

        Ok(rows)
    }

    pub fn find(&self, project_id: i32) -> Result<ProjectRecord, String> {
        self.find_optional(project_id)?
            .ok_or_else(|| format!("Project {} does not exist.", project_id))
    }

    pub fn find_optional(&self, project_id: i32) -> Result<Option<ProjectRecord>, String> {
        self.db
            .connection()
            .query_row(
                "select id, name, path, project_type from projects where id = ?1",
                params![project_id],
                |row| {
                    Ok(ProjectRecord {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        path: row.get(2)?,
                        project_type: row.get(3)?,
                    })
                },
            )
            .optional()
            .map_err(|error| error.to_string())
    }

    pub fn upsert(
        &self,
        id: Option<i32>,
        name: &str,
        path: &str,
        project_type: i32,
    ) -> Result<i32, String> {
        if let Some(id) = id {
            self.db
                .connection()
                .execute(
                    "update projects set name = ?1, path = ?2, project_type = ?3 where id = ?4",
                    params![name, path, project_type, id],
                )
                .map_err(|error| error.to_string())?;
            return Ok(id);
        }

        self.db
            .connection()
            .execute(
                "insert into projects (name, path, project_type) values (?1, ?2, ?3)",
                params![name, path, project_type],
            )
            .map_err(|error| error.to_string())?;

        Ok(self.db.connection().last_insert_rowid() as i32)
    }

    pub fn delete(&self, project_id: i32) -> Result<(), String> {
        let tx = self
            .db
            .connection()
            .unchecked_transaction()
            .map_err(|error| error.to_string())?;

        let deleted = tx
            .execute("delete from projects where id = ?1", params![project_id])
            .map_err(|error| error.to_string())?;

        if deleted == 0 {
            return Err(format!("Project {} does not exist.", project_id));
        }

        tx.execute(
            "delete from project_rule_bindings where project_id = ?1",
            params![project_id],
        )
        .map_err(|error| error.to_string())?;

        tx.commit().map_err(|error| error.to_string())?;
        Ok(())
    }
}
