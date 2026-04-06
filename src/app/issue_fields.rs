pub fn issue_custom_fields(issue: &api::models::Issue) -> Vec<(String, String)> {
    let Some(custom_fields) = &issue.custom_fields else {
        return Vec::new();
    };

    let mut out = Vec::new();
    for field in custom_fields {
        let (name, value) = issue_custom_field_parts(field);
        let name = name.map(|v| v.trim()).unwrap_or_default();
        if name.is_empty() {
            continue;
        }

        let value = value
            .map(custom_field_value_to_string)
            .unwrap_or_else(String::new);
        out.push((name.to_string(), value));
    }

    out.sort_by(|a, b| a.0.to_ascii_lowercase().cmp(&b.0.to_ascii_lowercase()));
    out
}

fn issue_custom_field_parts(
    field: &api::models::IssueCustomField,
) -> (Option<&String>, Option<&serde_json::Value>) {
    use api::models::IssueCustomField::*;

    match field {
        DateIssueCustomField { name, value, .. }
        | IssueCustomField { name, value, .. }
        | MultiBuildIssueCustomField { name, value, .. }
        | MultiEnumIssueCustomField { name, value, .. }
        | MultiGroupIssueCustomField { name, value, .. }
        | MultiOwnedIssueCustomField { name, value, .. }
        | MultiUserIssueCustomField { name, value, .. }
        | DatabaseMultiValueIssueCustomField { name, value, .. }
        | MultiVersionIssueCustomField { name, value, .. }
        | PeriodIssueCustomField { name, value, .. }
        | SimpleIssueCustomField { name, value, .. }
        | SingleBuildIssueCustomField { name, value, .. }
        | SingleEnumIssueCustomField { name, value, .. }
        | SingleGroupIssueCustomField { name, value, .. }
        | SingleOwnedIssueCustomField { name, value, .. }
        | SingleUserIssueCustomField { name, value, .. }
        | DatabaseSingleValueIssueCustomField { name, value, .. }
        | SingleVersionIssueCustomField { name, value, .. }
        | StateIssueCustomField { name, value, .. }
        | StateMachineIssueCustomField { name, value, .. }
        | TextIssueCustomField { name, value, .. } => (name.as_ref(), value.as_ref()),
    }
}

fn custom_field_value_to_string(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Null => String::new(),
        serde_json::Value::Bool(v) => v.to_string(),
        serde_json::Value::Number(v) => v.to_string(),
        serde_json::Value::String(v) => v.clone(),
        serde_json::Value::Array(values) => values
            .iter()
            .map(custom_field_value_to_string)
            .filter(|v| !v.is_empty())
            .collect::<Vec<_>>()
            .join(", "),
        serde_json::Value::Object(map) => {
            for key in [
                "name",
                "localizedName",
                "fullName",
                "login",
                "idReadable",
                "presentation",
                "text",
                "minutes",
                "id",
            ] {
                if let Some(v) = map.get(key) {
                    let text = custom_field_value_to_string(v);
                    if !text.is_empty() {
                        return text;
                    }
                }
            }
            serde_json::to_string(value).unwrap_or_default()
        }
    }
}
