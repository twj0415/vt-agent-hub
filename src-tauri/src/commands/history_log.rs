use crate::application::operation_service::OperationService;
use crate::infrastructure::database::Database;

pub struct CommandFailure<'a> {
    pub project_id: Option<i32>,
    pub tool_id: Option<i32>,
    pub related_rule_id: Option<i32>,
    pub kind: &'a str,
    pub title: &'a str,
    pub action: &'a str,
    pub detail: &'a str,
    pub related_path: Option<&'a str>,
    pub navigation_target: &'a str,
}

/// 命令层失败记录:目前临时打开 default DB(等 #8-5 命令接入 AppContainer 后会改为
/// 从 `tauri::State<AppContainer>` 拿 db,与业务写共享同一连接)。
pub fn record_command_failure(failure: CommandFailure<'_>) {
    if let Ok(db) = Database::open_default() {
        let _ = OperationService::record_failure(
            &db,
            failure.project_id,
            failure.tool_id,
            failure.related_rule_id,
            failure.kind,
            failure.title,
            failure.action,
            failure.detail,
            failure.related_path,
            failure.navigation_target,
        );
    }
}
