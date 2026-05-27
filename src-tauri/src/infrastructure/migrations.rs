use rusqlite::{params, Connection, OptionalExtension};

mod v001_init_core_tables;
mod v002_add_project_type;
mod v003_add_history_detail;
mod v004_add_rule_sort_order;
mod v005_expand_history_logs;
mod v006_add_resource_library_model;
mod v007_add_rule_version_sort_order;
mod v008_remove_legacy_rules_table;
mod v009_add_provider_switching;
mod v010_add_provider_credentials;
mod v011_enforce_provider_active_unique;
mod v012_introduce_direct_bindings;
mod v013_drop_legacy_pack_tables;

const LATEST_SCHEMA_VERSION: i32 = 13;

struct Migration {
    version: i32,
    name: &'static str,
    apply: fn(&Connection) -> Result<(), String>,
}

const MIGRATIONS: &[Migration] = &[
    Migration {
        version: 1,
        name: "001_init_core_tables",
        apply: v001_init_core_tables::apply,
    },
    Migration {
        version: 2,
        name: "002_add_project_type",
        apply: v002_add_project_type::apply,
    },
    Migration {
        version: 3,
        name: "003_add_history_detail",
        apply: v003_add_history_detail::apply,
    },
    Migration {
        version: 4,
        name: "004_add_rule_sort_order",
        apply: v004_add_rule_sort_order::apply,
    },
    Migration {
        version: 5,
        name: "005_expand_history_logs",
        apply: v005_expand_history_logs::apply,
    },
    Migration {
        version: 6,
        name: "006_add_resource_library_model",
        apply: v006_add_resource_library_model::apply,
    },
    Migration {
        version: 7,
        name: "007_add_rule_version_sort_order",
        apply: v007_add_rule_version_sort_order::apply,
    },
    Migration {
        version: 8,
        name: "008_remove_legacy_rules_table",
        apply: v008_remove_legacy_rules_table::apply,
    },
    Migration {
        version: 9,
        name: "009_add_provider_switching",
        apply: v009_add_provider_switching::apply,
    },
    Migration {
        version: 10,
        name: "010_add_provider_credentials",
        apply: v010_add_provider_credentials::apply,
    },
    Migration {
        version: 11,
        name: "011_enforce_provider_active_unique",
        apply: v011_enforce_provider_active_unique::apply,
    },
    Migration {
        version: 12,
        name: "012_introduce_direct_bindings",
        apply: v012_introduce_direct_bindings::apply,
    },
    Migration {
        version: 13,
        name: "013_drop_legacy_pack_tables",
        apply: v013_drop_legacy_pack_tables::apply,
    },
];

pub fn run(conn: &Connection) -> Result<(), String> {
    ensure_schema_meta(conn)?;
    let mut current_version = schema_version(conn)?;

    for migration in MIGRATIONS {
        if migration.version <= current_version {
            continue;
        }

        let tx = conn
            .unchecked_transaction()
            .map_err(|error| migration_error(migration, error.to_string()))?;

        (migration.apply)(&tx).map_err(|error| migration_error(migration, error))?;
        tx.execute(
            "insert or replace into schema_meta (key, value) values (?1, ?2)",
            params!["schema_version", migration.version.to_string()],
        )
        .map_err(|error| migration_error(migration, error.to_string()))?;
        tx.commit()
            .map_err(|error| migration_error(migration, error.to_string()))?;

        current_version = migration.version;
    }

    if current_version != LATEST_SCHEMA_VERSION {
        return Err(format!(
            "Database schema version {} is not supported. Latest supported version is {}.",
            current_version, LATEST_SCHEMA_VERSION
        ));
    }

    Ok(())
}

pub fn latest_schema_version() -> i32 {
    LATEST_SCHEMA_VERSION
}

fn ensure_schema_meta(conn: &Connection) -> Result<(), String> {
    conn.execute(
        "create table if not exists schema_meta (key text primary key, value text not null)",
        [],
    )
    .map_err(|error| error.to_string())?;
    Ok(())
}

fn schema_version(conn: &Connection) -> Result<i32, String> {
    let value = conn
        .query_row(
            "select value from schema_meta where key = ?1",
            params!["schema_version"],
            |row| row.get::<_, String>(0),
        )
        .optional()
        .map_err(|error| error.to_string())?;

    match value {
        Some(value) => value
            .parse::<i32>()
            .map_err(|_| format!("Invalid database schema_version '{}'.", value)),
        None => Ok(0),
    }
}

fn migration_error(migration: &Migration, error: String) -> String {
    format!(
        "Migration {} ({}) failed: {}",
        migration.version, migration.name, error
    )
}
pub(super) fn ensure_column(
    conn: &Connection,
    table: &str,
    column: &str,
    definition: &str,
) -> Result<(), String> {
    let sql = format!("pragma table_info({table})");
    let mut stmt = conn.prepare(&sql).map_err(|error| error.to_string())?;

    let exists = stmt
        .query_map([], |row| row.get::<_, String>(1))
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?
        .into_iter()
        .any(|name| name == column);

    if !exists {
        let alter = format!("alter table {table} add column {column} {definition}");
        conn.execute(&alter, [])
            .map_err(|error| error.to_string())?;
    }

    Ok(())
}

pub(super) fn table_exists(conn: &Connection, table: &str) -> Result<bool, String> {
    conn.query_row(
        "select count(*) from sqlite_master where type = 'table' and name = ?1",
        params![table],
        |row| row.get::<_, i32>(0),
    )
    .map(|count| count > 0)
    .map_err(|error| error.to_string())
}

pub(super) fn unique_rule_asset_key(
    conn: &Connection,
    base_key: &str,
    legacy_id: i32,
) -> Result<String, String> {
    let base_key = if base_key.trim().is_empty() {
        format!("legacy-rule-{legacy_id}")
    } else {
        base_key.to_string()
    };
    if !rule_asset_key_exists(conn, &base_key)? {
        return Ok(base_key);
    }

    let candidate = format!("{base_key}-{legacy_id}");
    if !rule_asset_key_exists(conn, &candidate)? {
        return Ok(candidate);
    }

    for index in 2..1000 {
        let candidate = format!("{base_key}-{legacy_id}-{index}");
        if !rule_asset_key_exists(conn, &candidate)? {
            return Ok(candidate);
        }
    }

    Err(format!(
        "No available rule asset key for legacy rule {legacy_id}."
    ))
}

fn rule_asset_key_exists(conn: &Connection, key: &str) -> Result<bool, String> {
    conn.query_row(
        "select count(*) from rule_assets where asset_key = ?1",
        params![key],
        |row| row.get::<_, i32>(0),
    )
    .map(|count| count > 0)
    .map_err(|error| error.to_string())
}

pub(super) fn asset_key(name: &str) -> String {
    let lowered = name.trim().to_lowercase();
    let mut key = String::new();
    let mut last_dash = false;

    for ch in lowered.chars() {
        if ch.is_ascii_alphanumeric() {
            key.push(ch);
            last_dash = false;
        } else if !last_dash {
            key.push('-');
            last_dash = true;
        }
    }

    key.trim_matches('-').to_string()
}
