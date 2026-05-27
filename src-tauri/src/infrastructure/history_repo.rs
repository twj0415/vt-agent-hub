use rusqlite::params;

use crate::infrastructure::database::Database;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistoryRecord {
    pub id: i32,
    pub project_id: Option<i32>,
    pub tool_id: Option<i32>,
    pub related_rule_id: Option<i32>,
    pub kind: String,
    pub title: String,
    pub created_at: String,
    pub action: String,
    pub result: String,
    pub result_code: i32,
    pub detail: String,
    pub related_path: String,
    pub navigation_target: String,
}

pub struct NewHistoryRecord<'a> {
    pub project_id: Option<i32>,
    pub tool_id: Option<i32>,
    pub related_rule_id: Option<i32>,
    pub kind: &'a str,
    pub title: &'a str,
    pub created_at: String,
    pub action: &'a str,
    pub result: &'a str,
    pub result_code: i32,
    pub detail: &'a str,
    pub related_path: String,
    pub navigation_target: &'a str,
}

pub struct HistoryRepo<'a> {
    db: &'a Database,
}

impl<'a> HistoryRepo<'a> {
    pub fn new(db: &'a Database) -> Self {
        Self { db }
    }

    pub fn list(&self) -> Result<Vec<HistoryRecord>, String> {
        let mut stmt = self
            .db
            .connection()
            .prepare("select id, project_id, tool_id, related_rule_id, kind, title, created_at, action, result, result_code, detail, related_path, navigation_target from history_logs order by id asc")
            .map_err(|error| error.to_string())?;

        let rows = stmt
            .query_map([], |row| {
                Ok(HistoryRecord {
                    id: row.get(0)?,
                    project_id: row.get(1)?,
                    tool_id: row.get(2)?,
                    related_rule_id: row.get(3)?,
                    kind: row.get(4)?,
                    title: row.get(5)?,
                    created_at: row.get(6)?,
                    action: row.get(7)?,
                    result: row.get(8)?,
                    result_code: row.get(9)?,
                    detail: row.get(10)?,
                    related_path: row.get(11)?,
                    navigation_target: row.get(12)?,
                })
            })
            .map_err(|error| error.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| error.to_string())?;

        Ok(rows)
    }

    pub fn latest_for_project(
        &self,
        project_id: i32,
        kind: &str,
    ) -> Result<Option<HistoryRecord>, String> {
        let sql = if kind == "any" {
            "select id, project_id, tool_id, related_rule_id, kind, title, created_at, action, result, result_code, detail, related_path, navigation_target from history_logs where project_id = ?1 order by id desc limit 1"
        } else {
            "select id, project_id, tool_id, related_rule_id, kind, title, created_at, action, result, result_code, detail, related_path, navigation_target from history_logs where project_id = ?1 and kind = ?2 order by id desc limit 1"
        };

        let mut stmt = self
            .db
            .connection()
            .prepare(sql)
            .map_err(|error| error.to_string())?;

        let result = if kind == "any" {
            stmt.query_row(params![project_id], |row| {
                Ok(HistoryRecord {
                    id: row.get(0)?,
                    project_id: row.get(1)?,
                    tool_id: row.get(2)?,
                    related_rule_id: row.get(3)?,
                    kind: row.get(4)?,
                    title: row.get(5)?,
                    created_at: row.get(6)?,
                    action: row.get(7)?,
                    result: row.get(8)?,
                    result_code: row.get(9)?,
                    detail: row.get(10)?,
                    related_path: row.get(11)?,
                    navigation_target: row.get(12)?,
                })
            })
        } else {
            stmt.query_row(params![project_id, kind], |row| {
                Ok(HistoryRecord {
                    id: row.get(0)?,
                    project_id: row.get(1)?,
                    tool_id: row.get(2)?,
                    related_rule_id: row.get(3)?,
                    kind: row.get(4)?,
                    title: row.get(5)?,
                    created_at: row.get(6)?,
                    action: row.get(7)?,
                    result: row.get(8)?,
                    result_code: row.get(9)?,
                    detail: row.get(10)?,
                    related_path: row.get(11)?,
                    navigation_target: row.get(12)?,
                })
            })
        };

        match result {
            Ok(record) => Ok(Some(record)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(error) => Err(error.to_string()),
        }
    }

    pub fn insert(&self, record: NewHistoryRecord<'_>) -> Result<(), String> {
        self.db
            .connection()
            .execute(
                "insert into history_logs (project_id, tool_id, related_rule_id, kind, title, created_at, action, result, result_code, detail, related_path, navigation_target) values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
                params![
                    record.project_id,
                    record.tool_id,
                    record.related_rule_id,
                    record.kind,
                    record.title,
                    record.created_at,
                    record.action,
                    record.result,
                    record.result_code,
                    record.detail,
                    record.related_path,
                    record.navigation_target
                ],
            )
            .map_err(|error| error.to_string())?;

        Ok(())
    }
}
