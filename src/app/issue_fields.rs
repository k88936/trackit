pub fn issue_custom_field(issue: &api::models::Issue, field_name: &str) -> String {
    let Some(custom_fields) = &issue.custom_fields else {
        return String::new();
    };

    for field in custom_fields {
        let (name, value) = issue_custom_field_parts(field);
        if name
            .map(|n| n.eq_ignore_ascii_case(field_name))
            .unwrap_or(false)
        {
            if let Some(value) = value {
                return custom_field_value_to_string(value);
            }
            return String::new();
        }
    }

    String::new()
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
            for key in ["name", "fullName", "login", "idReadable", "id"] {
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
