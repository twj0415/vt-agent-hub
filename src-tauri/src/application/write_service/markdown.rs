use crate::core::taxonomy;

#[derive(Debug, Default)]
pub struct MarkdownRuleParts {
    pub name: String,
    pub description: String,
    pub category_code: Option<i32>,
    pub body: String,
}

fn clean_frontmatter_value(value: &str) -> String {
    value
        .trim()
        .trim_matches('\"')
        .trim_matches('\'')
        .trim()
        .to_string()
}

fn parse_rule_category_value(value: &str) -> Option<i32> {
    taxonomy::parse_rule_category_alias(&clean_frontmatter_value(value))
}

fn parse_frontmatter_line(line: &str) -> Option<(String, String)> {
    let trimmed = line.trim();
    let (key, value) = trimmed
        .split_once(':')
        .or_else(|| trimmed.split_once('\u{ff1a}'))?;
    Some((key.trim().to_string(), clean_frontmatter_value(value)))
}

pub fn parse_markdown_rule(body: &str) -> MarkdownRuleParts {
    let normalized = body.trim_start_matches('\u{feff}').trim_start();
    let mut lines = normalized.lines().peekable();

    while let Some(line) = lines.peek() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("<!--") {
            lines.next();
            continue;
        }
        break;
    }

    if lines.next().map(str::trim) != Some("---") {
        return MarkdownRuleParts {
            body: normalized.to_string(),
            ..Default::default()
        };
    }

    let mut name = String::new();
    let mut description = String::new();
    let mut category_code = None;
    let mut body_lines = Vec::new();
    let mut in_body = false;

    for line in lines {
        let trimmed = line.trim();
        if !in_body {
            if trimmed == "---" {
                in_body = true;
                continue;
            }
            if let Some((key, value)) = parse_frontmatter_line(line) {
                match key.as_str() {
                    "name" => name = value,
                    "description" | "summary" => description = value,
                    "category" | "category_code" | "categoryCode" => {
                        category_code = parse_rule_category_value(&value)
                    }
                    _ => {}
                }
            }
            continue;
        }
        body_lines.push(line);
    }

    if !in_body {
        return MarkdownRuleParts {
            body: normalized.to_string(),
            ..Default::default()
        };
    }

    MarkdownRuleParts {
        name,
        description,
        category_code,
        body: body_lines.join("\n").trim_start().to_string(),
    }
}

pub fn markdown_description(body: &str) -> String {
    parse_markdown_rule(body).description
}
