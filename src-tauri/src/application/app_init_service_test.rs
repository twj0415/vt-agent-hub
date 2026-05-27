#[cfg(test)]
mod tests {
    use crate::application::app_init_service::AppInitService;
    use crate::core::tool_registry::CODEX_TOOL_ID;

    #[test]
    fn returns_codex_bootstrap_by_default() {
        let service = AppInitService::new().expect("service should initialize");
        let bootstrap = service
            .get_bootstrap()
            .expect("bootstrap should initialize the database");

        assert_eq!(bootstrap.app_name, "VT Hub Manager");
        assert_eq!(bootstrap.active_tool_id, CODEX_TOOL_ID);
    }
}
