#[derive(Debug, Clone, Copy)]
pub struct ToolCapabilitySet {
    pub rules: bool,
    pub presets: bool,
    pub credentials: bool,
    pub skill_install: bool,
    pub live_scan: bool,
    pub agents_output: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct ToolRegistryItem {
    pub id: i32,
    pub key: &'static str,
    pub enabled: bool,
    pub capabilities: ToolCapabilitySet,
}

pub const CODEX_TOOL_ID: i32 = 101;
pub const CLAUDE_TOOL_ID: i32 = 102;
pub const CURSOR_TOOL_ID: i32 = 103;

pub const TOOL_REGISTRY: [ToolRegistryItem; 3] = [
    ToolRegistryItem {
        id: CODEX_TOOL_ID,
        key: "codex",
        enabled: true,
        capabilities: ToolCapabilitySet {
            rules: true,
            presets: true,
            credentials: true,
            skill_install: true,
            live_scan: true,
            agents_output: true,
        },
    },
    ToolRegistryItem {
        id: CLAUDE_TOOL_ID,
        key: "claude",
        enabled: true,
        capabilities: ToolCapabilitySet {
            rules: true,
            presets: true,
            credentials: false,
            skill_install: false,
            live_scan: true,
            agents_output: true,
        },
    },
    ToolRegistryItem {
        id: CURSOR_TOOL_ID,
        key: "cursor",
        enabled: false,
        capabilities: ToolCapabilitySet {
            rules: true,
            presets: false,
            credentials: false,
            skill_install: false,
            live_scan: true,
            agents_output: true,
        },
    },
];

pub fn get_tool(id: i32) -> Option<ToolRegistryItem> {
    TOOL_REGISTRY.iter().find(|item| item.id == id).copied()
}
