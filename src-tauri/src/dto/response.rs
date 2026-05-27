use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppErrorDto {
    pub code: String,
    pub message: String,
    pub i18n_key: String,
    pub details: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<AppErrorDto>,
}

impl<T> AppResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn error(code: &str, message: &str, i18n_key: &str) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(AppErrorDto {
                code: code.to_string(),
                message: message.to_string(),
                i18n_key: i18n_key.to_string(),
                details: None,
            }),
        }
    }
}
