#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TaxonomyItem {
    pub code: i32,
    pub key: &'static str,
}

pub const RULE_CATEGORY_PERSONAL: i32 = 301;
pub const RULE_CATEGORY_PROJECT: i32 = 302;
pub const RULE_CATEGORY_BASE: i32 = 303;
pub const RULE_CATEGORY_STACK: i32 = 304;
pub const RULE_CATEGORY_CODE_QUALITY: i32 = 305;
pub const RULE_CATEGORY_GIT: i32 = 306;
pub const RULE_CATEGORY_DOMAIN: i32 = 307;
pub const RULE_CATEGORY_PROJECT_TYPE: i32 = 308;

pub const PROJECT_TYPE_WEB: i32 = 201;
pub const PROJECT_TYPE_MINI: i32 = 202;
pub const PROJECT_TYPE_DESKTOP: i32 = 203;

pub const SKILL_CATEGORY_CODING: i32 = 401;
pub const SKILL_CATEGORY_UI_DESIGN: i32 = 402;

pub const DEFAULT_RULE_CATEGORY: i32 = RULE_CATEGORY_STACK;
pub const DEFAULT_PROJECT_TYPE: i32 = PROJECT_TYPE_WEB;
pub const DEFAULT_SKILL_CATEGORY: i32 = SKILL_CATEGORY_CODING;

pub const RULE_CATEGORIES: &[TaxonomyItem] = &[
    TaxonomyItem {
        code: RULE_CATEGORY_PERSONAL,
        key: "personal",
    },
    TaxonomyItem {
        code: RULE_CATEGORY_PROJECT,
        key: "project",
    },
    TaxonomyItem {
        code: RULE_CATEGORY_BASE,
        key: "base",
    },
    TaxonomyItem {
        code: RULE_CATEGORY_STACK,
        key: "stack",
    },
    TaxonomyItem {
        code: RULE_CATEGORY_CODE_QUALITY,
        key: "code-quality",
    },
    TaxonomyItem {
        code: RULE_CATEGORY_GIT,
        key: "git",
    },
    TaxonomyItem {
        code: RULE_CATEGORY_DOMAIN,
        key: "domain",
    },
    TaxonomyItem {
        code: RULE_CATEGORY_PROJECT_TYPE,
        key: "project-type",
    },
];

pub const PROJECT_TYPES: &[TaxonomyItem] = &[
    TaxonomyItem {
        code: PROJECT_TYPE_WEB,
        key: "web",
    },
    TaxonomyItem {
        code: PROJECT_TYPE_MINI,
        key: "mini",
    },
    TaxonomyItem {
        code: PROJECT_TYPE_DESKTOP,
        key: "desktop",
    },
];

pub const SKILL_CATEGORIES: &[TaxonomyItem] = &[
    TaxonomyItem {
        code: SKILL_CATEGORY_CODING,
        key: "coding",
    },
    TaxonomyItem {
        code: SKILL_CATEGORY_UI_DESIGN,
        key: "ui-design",
    },
];

pub fn is_rule_category_code(value: i32) -> bool {
    RULE_CATEGORIES.iter().any(|item| item.code == value)
}

pub fn is_project_type_code(value: i32) -> bool {
    PROJECT_TYPES.iter().any(|item| item.code == value)
}

pub fn is_skill_category_code(value: i32) -> bool {
    SKILL_CATEGORIES.iter().any(|item| item.code == value)
}

pub fn parse_rule_category_alias(value: &str) -> Option<i32> {
    match normalize_alias(value).as_str() {
        "301" | "personal" => Some(RULE_CATEGORY_PERSONAL),
        "302" | "project" => Some(RULE_CATEGORY_PROJECT),
        "303" | "base" => Some(RULE_CATEGORY_BASE),
        "304" | "stack" => Some(RULE_CATEGORY_STACK),
        "305" | "codequality" | "code-quality" | "code_quality" => Some(RULE_CATEGORY_CODE_QUALITY),
        "306" | "git" => Some(RULE_CATEGORY_GIT),
        "307" | "domain" => Some(RULE_CATEGORY_DOMAIN),
        "308" | "projecttype" | "project-type" | "project_type" => Some(RULE_CATEGORY_PROJECT_TYPE),
        _ => None,
    }
}

fn normalize_alias(value: &str) -> String {
    value
        .trim()
        .trim_matches('"')
        .trim_matches('\'')
        .trim()
        .to_ascii_lowercase()
}
