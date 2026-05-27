#[macro_export]
macro_rules! wrap_command {
    (
        service: $service:expr,
        call: $call:expr,
        error_code: $error_code:expr,
        i18n: $i18n:expr $(,)?
    ) => {{
        let service = $service;
        match $call(service) {
            Ok(value) => $crate::dto::AppResponse::success(value),
            Err(error) => $crate::dto::AppResponse::error($error_code, &error, $i18n),
        }
    }};
    (
        service: $service:expr,
        call: $call:expr,
        error_code: $error_code:expr,
        i18n: $i18n:expr,
        history: {
            kind: $kind:expr,
            title: $title:expr,
            action: $action:expr,
            route: $route:expr $(,
            related_path: $related_path:expr)? $(,)?
        } $(,)?
    ) => {{
        let service = $service;
        match $call(service) {
            Ok(value) => $crate::dto::AppResponse::success(value),
            Err(error) => {
                $crate::commands::history_log::record_command_failure(
                    $crate::commands::history_log::CommandFailure {
                        project_id: None,
                        tool_id: None,
                        related_rule_id: None,
                        kind: $kind,
                        title: $title,
                        action: $action,
                        detail: &error,
                        related_path: $crate::wrap_command!(@related_path $($related_path)?),
                        navigation_target: $route,
                    },
                );
                $crate::dto::AppResponse::error($error_code, &error, $i18n)
            }
        }
    }};
    (@related_path) => { None };
    (@related_path $related_path:expr) => { Some($related_path) };
}
