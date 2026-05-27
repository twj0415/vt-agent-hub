use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingItemDto {
    pub id: i32,
    pub name: String,
    pub value: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsPathDto {
    pub key: String,
    pub path: String,
    pub note: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsTruthSourceDto {
    pub key: String,
    pub canonical: String,
    pub mirrors: Vec<String>,
    pub note: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsSnapshotDto {
    pub items: Vec<SettingItemDto>,
    pub paths: Vec<SettingsPathDto>,
    pub truth_sources: Vec<SettingsTruthSourceDto>,
}
