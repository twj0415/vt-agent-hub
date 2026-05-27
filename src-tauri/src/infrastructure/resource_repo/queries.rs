use super::{ResourceRepo, RuleVersionRecord, SkillVersionRecord};

impl<'a> ResourceRepo<'a> {
    pub fn list_rule_versions(&self) -> Result<Vec<RuleVersionRecord>, String> {
        let mut stmt = self
            .db
            .connection()
            .prepare(
                r#"
                select
                    rule_assets.id,
                    rule_assets.asset_key,
                    rule_versions.id,
                    rule_versions.version_no,
                    rule_versions.code,
                    rule_versions.name,
                    rule_versions.category_code,
                    rule_versions.sort_order,
                    rule_versions.state,
                    rule_versions.summary,
                    rule_versions.body
                from rule_versions
                inner join rule_assets on rule_assets.id = rule_versions.rule_asset_id
                order by rule_versions.category_code asc, rule_versions.name asc, rule_versions.version_no desc
                "#,
            )
            .map_err(|error| error.to_string())?;

        let rows = stmt
            .query_map([], |row| {
                Ok(RuleVersionRecord {
                    asset_id: row.get(0)?,
                    asset_key: row.get(1)?,
                    version_id: row.get(2)?,
                    version_no: row.get(3)?,
                    code: row.get(4)?,
                    name: row.get(5)?,
                    category_code: row.get(6)?,
                    sort_order: row.get(7)?,
                    state: row.get(8)?,
                    summary: row.get(9)?,
                    body: row.get(10)?,
                })
            })
            .map_err(|error| error.to_string())?;

        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|error| error.to_string())
    }

    pub fn list_latest_rule_versions(&self) -> Result<Vec<RuleVersionRecord>, String> {
        let mut stmt = self
            .db
            .connection()
            .prepare(
                r#"
                select
                    rule_assets.id,
                    rule_assets.asset_key,
                    rule_versions.id,
                    rule_versions.version_no,
                    rule_versions.code,
                    rule_versions.name,
                    rule_versions.category_code,
                    rule_versions.sort_order,
                    rule_versions.state,
                    rule_versions.summary,
                    rule_versions.body
                from rule_versions
                inner join rule_assets on rule_assets.id = rule_versions.rule_asset_id
                inner join (
                    select rule_asset_id, max(version_no) as max_version_no
                    from rule_versions
                    group by rule_asset_id
                ) latest on latest.rule_asset_id = rule_versions.rule_asset_id and latest.max_version_no = rule_versions.version_no
                order by rule_versions.category_code asc, rule_versions.name asc
                "#,
            )
            .map_err(|error| error.to_string())?;

        let rows = stmt
            .query_map([], |row| {
                Ok(RuleVersionRecord {
                    asset_id: row.get(0)?,
                    asset_key: row.get(1)?,
                    version_id: row.get(2)?,
                    version_no: row.get(3)?,
                    code: row.get(4)?,
                    name: row.get(5)?,
                    category_code: row.get(6)?,
                    sort_order: row.get(7)?,
                    state: row.get(8)?,
                    summary: row.get(9)?,
                    body: row.get(10)?,
                })
            })
            .map_err(|error| error.to_string())?;

        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|error| error.to_string())
    }

    pub fn list_skill_versions(&self) -> Result<Vec<SkillVersionRecord>, String> {
        let mut stmt = self
            .db
            .connection()
            .prepare(
                r#"
                select
                    skill_assets.id,
                    skill_assets.asset_key,
                    skill_versions.id,
                    skill_versions.version_no,
                    skill_versions.code,
                    skill_versions.name,
                    skill_versions.category_code,
                    skill_versions.state,
                    skill_versions.summary,
                    skill_versions.body
                from skill_versions
                inner join skill_assets on skill_assets.id = skill_versions.skill_asset_id
                order by skill_versions.name asc, skill_versions.version_no desc
                "#,
            )
            .map_err(|error| error.to_string())?;

        let rows = stmt
            .query_map([], |row| {
                Ok(SkillVersionRecord {
                    asset_id: row.get(0)?,
                    asset_key: row.get(1)?,
                    version_id: row.get(2)?,
                    version_no: row.get(3)?,
                    code: row.get(4)?,
                    name: row.get(5)?,
                    category_code: row.get(6)?,
                    state: row.get(7)?,
                    summary: row.get(8)?,
                    body: row.get(9)?,
                })
            })
            .map_err(|error| error.to_string())?;

        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|error| error.to_string())
    }

    pub fn list_latest_skill_versions(&self) -> Result<Vec<SkillVersionRecord>, String> {
        let mut stmt = self
            .db
            .connection()
            .prepare(
                r#"
                select
                    skill_assets.id,
                    skill_assets.asset_key,
                    skill_versions.id,
                    skill_versions.version_no,
                    skill_versions.code,
                    skill_versions.name,
                    skill_versions.category_code,
                    skill_versions.state,
                    skill_versions.summary,
                    skill_versions.body
                from skill_versions
                inner join skill_assets on skill_assets.id = skill_versions.skill_asset_id
                inner join (
                    select skill_asset_id, max(version_no) as max_version_no
                    from skill_versions
                    group by skill_asset_id
                ) latest on latest.skill_asset_id = skill_versions.skill_asset_id and latest.max_version_no = skill_versions.version_no
                order by skill_versions.name asc
                "#,
            )
            .map_err(|error| error.to_string())?;

        let rows = stmt
            .query_map([], |row| {
                Ok(SkillVersionRecord {
                    asset_id: row.get(0)?,
                    asset_key: row.get(1)?,
                    version_id: row.get(2)?,
                    version_no: row.get(3)?,
                    code: row.get(4)?,
                    name: row.get(5)?,
                    category_code: row.get(6)?,
                    state: row.get(7)?,
                    summary: row.get(8)?,
                    body: row.get(9)?,
                })
            })
            .map_err(|error| error.to_string())?;

        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|error| error.to_string())
    }

    pub fn find_rule_version(&self, version_id: i32) -> Result<RuleVersionRecord, String> {
        self.list_rule_versions()?
            .into_iter()
            .find(|row| row.version_id == version_id)
            .ok_or_else(|| format!("Rule version {} does not exist.", version_id))
    }

    pub fn find_skill_version(&self, version_id: i32) -> Result<SkillVersionRecord, String> {
        self.list_skill_versions()?
            .into_iter()
            .find(|row| row.version_id == version_id)
            .ok_or_else(|| format!("Skill version {} does not exist.", version_id))
    }

    pub fn find_latest_skill_version_by_asset(
        &self,
        asset_id: i32,
    ) -> Result<SkillVersionRecord, String> {
        self.list_latest_skill_versions()?
            .into_iter()
            .find(|row| row.asset_id == asset_id)
            .ok_or_else(|| format!("Skill asset {} has no versions.", asset_id))
    }

    pub fn find_latest_rule_version_by_asset(
        &self,
        asset_id: i32,
    ) -> Result<RuleVersionRecord, String> {
        self.list_latest_rule_versions()?
            .into_iter()
            .find(|row| row.asset_id == asset_id)
            .ok_or_else(|| format!("Rule asset {} has no versions.", asset_id))
    }

    pub fn find_latest_rule_version_by_name(
        &self,
        name: &str,
    ) -> Result<Option<RuleVersionRecord>, String> {
        Ok(self
            .list_latest_rule_versions()?
            .into_iter()
            .find(|row| row.name.eq_ignore_ascii_case(name)))
    }

    pub fn find_latest_skill_version_by_name(
        &self,
        name: &str,
    ) -> Result<Option<SkillVersionRecord>, String> {
        Ok(self
            .list_latest_skill_versions()?
            .into_iter()
            .find(|row| row.name.eq_ignore_ascii_case(name)))
    }
}
