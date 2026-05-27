#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleVersionRecord {
    pub asset_id: i32,
    pub asset_key: String,
    pub version_id: i32,
    pub version_no: i32,
    pub code: i32,
    pub name: String,
    pub category_code: i32,
    pub sort_order: i32,
    pub state: i32,
    pub summary: String,
    pub body: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillVersionRecord {
    pub asset_id: i32,
    pub asset_key: String,
    pub version_id: i32,
    pub version_no: i32,
    pub code: i32,
    pub name: String,
    pub category_code: i32,
    pub state: i32,
    pub summary: String,
    pub body: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackItemRecord {
    pub item_type: String,
    pub asset_id: i32,
    pub asset_version_id: i32,
    pub asset_version_no: i32,
    pub sort_order: i32,
    pub required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectRulePackBindingRecord {
    pub project_id: i32,
    pub tool_id: Option<i32>,
    pub pack_id: i32,
    pub pack_name: String,
    pub pack_type: String,
    pub pack_version_id: i32,
    pub pack_version_no: i32,
    pub update_policy: String,
    pub enabled: bool,
    pub items: Vec<PackItemRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolRulePackBindingRecord {
    pub tool_id: i32,
    pub pack_id: i32,
    pub pack_name: String,
    pub pack_type: String,
    pub pack_version_id: i32,
    pub pack_version_no: i32,
    pub update_policy: String,
    pub enabled: bool,
    pub items: Vec<PackItemRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolSkillInstallRecord {
    pub tool_id: i32,
    pub skill_asset_id: i32,
    pub required_version_id: Option<i32>,
    pub installed_version_id: Option<i32>,
    pub state: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleImpactRecord {
    pub rule_asset_id: i32,
    pub rule_name: String,
    pub project_names: Vec<String>,
    pub tool_ids: Vec<i32>,
    pub project_tool_ids: Vec<i32>,
    pub global_tool_ids: Vec<i32>,
}
