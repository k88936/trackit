use crate::error::{Result, TrackItError};

pub fn build_issue_query(project: Option<&str>, filters: &[(String, String)]) -> Option<String> {
    let mut parts = Vec::new();

    if let Some(project) = project {
        parts.push(format!("project:{}", quote_query_value(project)));
    }

    for (key, value) in filters {
        parts.push(format!(
            "{}:{}",
            quote_query_field_name(key),
            quote_query_value(value)
        ));
    }

    if parts.is_empty() {
        None
    } else {
        Some(parts.join(" "))
    }
}

pub fn parse_key_value_specs(values: &[String], flag_name: &str) -> Result<Vec<(String, String)>> {
    values
        .iter()
        .map(|value| parse_key_value_spec(value, flag_name))
        .collect()
}

pub fn parse_link_spec(value: &str) -> Result<(String, String)> {
    let Some((relation, issue)) = value.split_once(':') else {
        return Err(TrackItError::Config(format!(
            "Invalid link format '{value}'. Expected RELATION:ISSUE, e.g. 'relates to:PRJ-123'"
        )));
    };

    let relation = relation.trim();
    let issue = issue.trim();
    if relation.is_empty() || issue.is_empty() {
        return Err(TrackItError::Config(format!(
            "Invalid link format '{value}'. Both relation and issue must be non-empty"
        )));
    }

    Ok((relation.to_string(), issue.to_string()))
}

pub fn summarize_plain_values(values: &[String]) -> String {
    if values.is_empty() {
        return "(none)".to_string();
    }
    let limit = 12usize;
    if values.len() > limit {
        format!(
            "{} ... (+{} more)",
            values[..limit].join(", "),
            values.len() - limit
        )
    } else {
        values.join(", ")
    }
}

fn parse_key_value_spec(value: &str, flag_name: &str) -> Result<(String, String)> {
    let Some((key, raw_value)) = value.split_once('=') else {
        return Err(TrackItError::Config(format!(
            "Invalid {flag_name} format '{value}'. Expected KEY=VALUE"
        )));
    };

    let key = key.trim();
    let parsed_value = raw_value.trim();
    if key.is_empty() || parsed_value.is_empty() {
        return Err(TrackItError::Config(format!(
            "Invalid {flag_name} format '{value}'. Both KEY and VALUE must be non-empty"
        )));
    }

    Ok((key.to_string(), parsed_value.to_string()))
}

fn quote_query_field_name(value: &str) -> String {
    if value.chars().any(|c| c.is_whitespace() || c == '"') {
        format!("\"{}\"", value.replace('"', "\\\""))
    } else {
        value.to_string()
    }
}

fn quote_query_value(value: &str) -> String {
    if value.chars().any(char::is_whitespace) {
        format!("\"{}\"", value.replace('"', "\\\""))
    } else {
        value.to_string()
    }
}
