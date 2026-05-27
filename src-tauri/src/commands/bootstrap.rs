use crate::application::app_container::AppContainer;
use crate::application::app_init_service::AppInitService;
use crate::dto::{AppBootstrapDto, AppResponse, AppStateDto};

#[tauri::command]
pub fn get_app_bootstrap(state: tauri::State<'_, AppContainer>) -> AppResponse<AppBootstrapDto> {
    let service = AppInitService::with_container(state.inner());
    match service.get_bootstrap() {
        Ok(bootstrap) => AppResponse::success(AppBootstrapDto {
            app_name: bootstrap.app_name,
            state: AppStateDto::Planned,
            active_tool_id: bootstrap.active_tool_id,
        }),
        Err(error) => {
            AppResponse::error("app_bootstrap_failed", &error, "errors.appBootstrapFailed")
        }
    }
}

#[tauri::command]
pub fn reset_app_data(
    state: tauri::State<'_, AppContainer>,
    confirm_risk: bool,
) -> AppResponse<String> {
    if !confirm_risk {
        return AppResponse::error(
            "app_reset_not_confirmed",
            "Reset confirmation is required.",
            "errors.appResetNotConfirmed",
        );
    }

    match AppInitService::reset_app_data_with_container(state.inner()) {
        Ok(message) => AppResponse::success(message),
        Err(error) => AppResponse::error("app_reset_failed", &error, "errors.appResetFailed"),
    }
}
