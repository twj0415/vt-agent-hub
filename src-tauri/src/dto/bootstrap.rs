use serde::Serialize;

#[derive(Debug, Serialize)]
pub enum AppStateDto {
    Planned,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppBootstrapDto {
    pub app_name: String,
    pub state: AppStateDto,
    pub active_tool_id: i32,
}
