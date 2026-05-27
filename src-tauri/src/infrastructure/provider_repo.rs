use rusqlite::OptionalExtension;
use serde_json::Value;

use crate::infrastructure::credential_store::CredentialStore;
use crate::infrastructure::database::Database;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderRecord {
    pub id: i32,
    pub name: String,
    pub category: String,
    pub website: String,
    pub note: String,
    pub sort_order: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProviderToolConfigRecord {
    pub id: i32,
    pub provider_id: i32,
    pub tool_id: i32,
    pub schema_version: i32,
    pub display_name: String,
    pub model: String,
    pub reasoning: String,
    pub base_url: String,
    pub credential_ref: String,
    pub config_json: Value,
    pub is_active: bool,
    pub state: i32,
    pub last_check_status: String,
    pub last_check_latency_ms: Option<i32>,
    pub last_check_message: String,
    pub last_checked_at: String,
}

#[derive(Debug, Clone)]
pub struct ProviderWithConfigs {
    pub provider: ProviderRecord,
    pub configs: Vec<ProviderToolConfigRecord>,
}

#[derive(Debug, Clone)]
pub struct ProviderConfigUpsert {
    pub id: Option<i32>,
    pub tool_id: i32,
    pub schema_version: i32,
    pub display_name: String,
    pub model: String,
    pub reasoning: String,
    pub base_url: String,
    pub credential_ref: String,
    pub config_json: Value,
}

pub struct ProviderRepo<'a> {
    db: &'a Database,
}

impl<'a> ProviderRepo<'a> {
    pub fn new(db: &'a Database) -> Self {
        Self { db }
    }

    pub fn list(&self, tool_id: Option<i32>) -> Result<Vec<ProviderWithConfigs>, String> {
        let providers = self.list_provider_records()?;
        let mut result = Vec::new();

        for provider in providers {
            let configs = self.list_configs(provider.id, tool_id)?;
            if tool_id.is_some() && configs.is_empty() {
                continue;
            }
            result.push(ProviderWithConfigs { provider, configs });
        }

        Ok(result)
    }

    pub fn find_provider(&self, id: i32) -> Result<ProviderRecord, String> {
        self.db
            .connection()
            .query_row(
                "select id, name, category, website, note, sort_order from providers where id = ?1",
                rusqlite::params![id],
                Self::map_provider,
            )
            .map_err(|error| error.to_string())
    }

    pub fn find_config(&self, config_id: i32) -> Result<ProviderToolConfigRecord, String> {
        self.db
            .connection()
            .query_row(
                "select id, provider_id, tool_id, schema_version, display_name, model, reasoning, base_url, credential_ref, config_json, is_active, state, last_check_status, last_check_latency_ms, last_check_message, last_checked_at from provider_tool_configs where id = ?1",
                rusqlite::params![config_id],
                Self::map_config,
            )
            .map_err(|error| error.to_string())
    }

    pub fn upsert_provider(
        &self,
        id: Option<i32>,
        name: &str,
        category: &str,
        website: &str,
        note: &str,
        configs: &[ProviderConfigUpsert],
    ) -> Result<i32, String> {
        let tx = self
            .db
            .connection()
            .unchecked_transaction()
            .map_err(|error| error.to_string())?;

        let provider_id = if let Some(id) = id {
            let changed = tx
                .execute(
                    "update providers set name = ?1, category = ?2, website = ?3, note = ?4, updated_at = current_timestamp where id = ?5",
                    rusqlite::params![name, category, website, note, id],
                )
                .map_err(|error| error.to_string())?;
            if changed == 0 {
                return Err(format!("Provider {} does not exist.", id));
            }
            id
        } else {
            tx.execute(
                "insert into providers (name, category, website, note, sort_order) values (?1, ?2, ?3, ?4, ?5)",
                rusqlite::params![name, category, website, note, self.next_sort_order()?],
            )
            .map_err(|error| error.to_string())?;
            tx.last_insert_rowid() as i32
        };

        for config in configs {
            let config_json =
                serde_json::to_string(&config.config_json).map_err(|error| error.to_string())?;
            if let Some(config_id) = config.id {
                let changed = tx
                    .execute(
                        "update provider_tool_configs set tool_id = ?1, schema_version = ?2, display_name = ?3, model = ?4, reasoning = ?5, base_url = ?6, credential_ref = ?7, config_json = ?8, updated_at = current_timestamp where id = ?9 and provider_id = ?10",
                        rusqlite::params![
                            config.tool_id,
                            config.schema_version,
                            config.display_name,
                            config.model,
                            config.reasoning,
                            config.base_url,
                            config.credential_ref,
                            config_json,
                            config_id,
                            provider_id,
                        ],
                    )
                    .map_err(|error| error.to_string())?;
                if changed == 0 {
                    return Err(format!("Provider config {} does not exist.", config_id));
                }
            } else {
                tx.execute(
                    "insert into provider_tool_configs (provider_id, tool_id, schema_version, display_name, model, reasoning, base_url, credential_ref, config_json, is_active, state) values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 0, 504)",
                    rusqlite::params![
                        provider_id,
                        config.tool_id,
                        config.schema_version,
                        config.display_name,
                        config.model,
                        config.reasoning,
                        config.base_url,
                        config.credential_ref,
                        config_json,
                    ],
                )
                .map_err(|error| error.to_string())?;
            }
        }

        tx.commit().map_err(|error| error.to_string())?;
        Ok(provider_id)
    }

    pub fn delete_provider(&self, provider_id: i32) -> Result<(), String> {
        let active_count: i32 = self
            .db
            .connection()
            .query_row(
                "select count(*) from provider_tool_configs where provider_id = ?1 and is_active = 1",
                rusqlite::params![provider_id],
                |row| row.get(0),
            )
            .map_err(|error| error.to_string())?;
        if active_count > 0 {
            return Err(
                "Active provider cannot be deleted. Switch to another provider first.".to_string(),
            );
        }

        // 删 DB 前先记录所有凭证引用,事务提交后再 best-effort 清理 OS 凭证存储;
        // 凭证清理失败不应回滚 DB,避免出现"DB 删了但报错"的不一致。
        let credential_refs = self.collect_credential_refs(provider_id)?;

        let tx = self
            .db
            .connection()
            .unchecked_transaction()
            .map_err(|error| error.to_string())?;
        tx.execute(
            "delete from provider_tool_configs where provider_id = ?1",
            rusqlite::params![provider_id],
        )
        .map_err(|error| error.to_string())?;
        let deleted = tx
            .execute(
                "delete from providers where id = ?1",
                rusqlite::params![provider_id],
            )
            .map_err(|error| error.to_string())?;
        if deleted == 0 {
            return Err(format!("Provider {} does not exist.", provider_id));
        }
        tx.commit().map_err(|error| error.to_string())?;

        for credential_ref in credential_refs {
            let _ = CredentialStore::clear_provider_token(&credential_ref);
        }

        Ok(())
    }

    fn collect_credential_refs(&self, provider_id: i32) -> Result<Vec<String>, String> {
        let mut stmt = self
            .db
            .connection()
            .prepare(
                "select credential_ref from provider_tool_configs \
                 where provider_id = ?1 and credential_ref != ''",
            )
            .map_err(|error| error.to_string())?;
        let rows = stmt
            .query_map(rusqlite::params![provider_id], |row| {
                row.get::<_, String>(0)
            })
            .map_err(|error| error.to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|error| error.to_string())
    }

    pub fn duplicate_provider(&self, provider_id: i32) -> Result<i32, String> {
        let provider = self.find_provider(provider_id)?;
        let configs = self.list_configs(provider_id, None)?;
        // 复制供应商时不复用 credential_ref,避免两个供应商共享同一份凭证;
        // 用户需要在复制后重新填写 token。
        let cloned = configs
            .into_iter()
            .map(|config| ProviderConfigUpsert {
                id: None,
                tool_id: config.tool_id,
                schema_version: config.schema_version,
                display_name: config.display_name,
                model: config.model,
                reasoning: config.reasoning,
                base_url: config.base_url,
                credential_ref: String::new(),
                config_json: config.config_json,
            })
            .collect::<Vec<_>>();

        self.upsert_provider(
            None,
            &format!("{} Copy", provider.name),
            &provider.category,
            &provider.website,
            &provider.note,
            &cloned,
        )
    }

    pub fn activate_config(&self, tool_id: i32, config_id: i32) -> Result<(), String> {
        let tx = self
            .db
            .connection()
            .unchecked_transaction()
            .map_err(|error| error.to_string())?;
        let exists = tx
            .query_row(
                "select 1 from provider_tool_configs where id = ?1 and tool_id = ?2",
                rusqlite::params![config_id, tool_id],
                |_| Ok(()),
            )
            .optional()
            .map_err(|error| error.to_string())?
            .is_some();
        if !exists {
            return Err(format!(
                "Provider config {} does not exist for tool {}.",
                config_id, tool_id
            ));
        }

        tx.execute(
            "update provider_tool_configs set is_active = case when id = ?1 then 1 else 0 end, state = case when id = ?1 then 502 else 504 end where tool_id = ?2",
            rusqlite::params![config_id, tool_id],
        )
        .map_err(|error| error.to_string())?;
        tx.commit().map_err(|error| error.to_string())?;
        Ok(())
    }

    fn list_provider_records(&self) -> Result<Vec<ProviderRecord>, String> {
        let mut stmt = self
            .db
            .connection()
            .prepare(
                "select id, name, category, website, note, sort_order from providers order by sort_order asc, id asc",
            )
            .map_err(|error| error.to_string())?;
        let rows = stmt
            .query_map([], Self::map_provider)
            .map_err(|error| error.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| error.to_string())?;
        Ok(rows)
    }

    fn list_configs(
        &self,
        provider_id: i32,
        tool_id: Option<i32>,
    ) -> Result<Vec<ProviderToolConfigRecord>, String> {
        let sql = if tool_id.is_some() {
            "select id, provider_id, tool_id, schema_version, display_name, model, reasoning, base_url, credential_ref, config_json, is_active, state, last_check_status, last_check_latency_ms, last_check_message, last_checked_at from provider_tool_configs where provider_id = ?1 and tool_id = ?2 order by id asc"
        } else {
            "select id, provider_id, tool_id, schema_version, display_name, model, reasoning, base_url, credential_ref, config_json, is_active, state, last_check_status, last_check_latency_ms, last_check_message, last_checked_at from provider_tool_configs where provider_id = ?1 order by tool_id asc, id asc"
        };
        let mut stmt = self
            .db
            .connection()
            .prepare(sql)
            .map_err(|error| error.to_string())?;
        let mapper = |row: &rusqlite::Row<'_>| Self::map_config(row);
        let rows = if let Some(tool_id) = tool_id {
            stmt.query_map(rusqlite::params![provider_id, tool_id], mapper)
                .map_err(|error| error.to_string())?
                .collect::<Result<Vec<_>, _>>()
        } else {
            stmt.query_map(rusqlite::params![provider_id], mapper)
                .map_err(|error| error.to_string())?
                .collect::<Result<Vec<_>, _>>()
        };

        rows.map_err(|error| error.to_string())
    }

    fn next_sort_order(&self) -> Result<i32, String> {
        self.db
            .connection()
            .query_row(
                "select coalesce(max(sort_order), 0) + 10 from providers",
                [],
                |row| row.get(0),
            )
            .map_err(|error| error.to_string())
    }

    fn map_provider(row: &rusqlite::Row<'_>) -> rusqlite::Result<ProviderRecord> {
        Ok(ProviderRecord {
            id: row.get(0)?,
            name: row.get(1)?,
            category: row.get(2)?,
            website: row.get(3)?,
            note: row.get(4)?,
            sort_order: row.get(5)?,
        })
    }

    fn map_config(row: &rusqlite::Row<'_>) -> rusqlite::Result<ProviderToolConfigRecord> {
        let config_json_text: String = row.get(9)?;
        let config_json = serde_json::from_str(&config_json_text).unwrap_or(Value::Null);
        Ok(ProviderToolConfigRecord {
            id: row.get(0)?,
            provider_id: row.get(1)?,
            tool_id: row.get(2)?,
            schema_version: row.get(3)?,
            display_name: row.get(4)?,
            model: row.get(5)?,
            reasoning: row.get(6)?,
            base_url: row.get(7)?,
            credential_ref: row.get(8)?,
            config_json,
            is_active: row.get::<_, i32>(10)? == 1,
            state: row.get(11)?,
            last_check_status: row.get(12)?,
            last_check_latency_ms: row.get(13)?,
            last_check_message: row.get(14)?,
            last_checked_at: row.get(15)?,
        })
    }
}
