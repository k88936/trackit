use crate::error::{Result, TrackItError};
use crate::utils::text;

pub fn build_issue_query(project: Option<&str>, filters: &[(String, String)]) -> Option<String> {
    let mut parts = Vec::new();

    if let Some(project) = project {
        parts.push(format!("project:{}", text::quote_query_value(project)));
    }

    for (key, value) in filters {
        parts.push(format!(
            "{}:{}",
            text::quote_query_field_name(key),
            text::quote_query_value(value)
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

#[cfg(test)]
mod tests {
    use super::build_issue_query;

    #[test]
    fn builds_query_with_braced_value_for_spaces() {
        let query = build_issue_query(Some("TAL"), &[("State".to_string(), "To do".to_string())]);
        assert_eq!(query.as_deref(), Some("project:TAL State:{To do}"));
    }

    #[test]
    fn keeps_non_spaced_value_unwrapped() {
        let query = build_issue_query(
            Some("TAL"),
            &[("Priority".to_string(), "Normal".to_string())],
        );
        assert_eq!(query.as_deref(), Some("project:TAL Priority:Normal"));
    }
}
