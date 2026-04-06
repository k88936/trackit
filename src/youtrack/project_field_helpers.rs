use api::models;

#[derive(Clone, Debug, serde::Serialize)]
pub struct ProjectFieldSuggestion {
    pub name: String,
    pub values: Vec<String>,
}

pub(super) fn project_custom_field_id(field: &models::ProjectCustomField) -> Option<&str> {
    use models::ProjectCustomField::*;

    match field {
        BuildProjectCustomField { id, .. }
        | BundleProjectCustomField { id, .. }
        | EnumProjectCustomField { id, .. }
        | GroupProjectCustomField { id, .. }
        | OwnedProjectCustomField { id, .. }
        | PeriodProjectCustomField { id, .. }
        | ProjectCustomField { id, .. }
        | SimpleProjectCustomField { id, .. }
        | StateProjectCustomField { id, .. }
        | TextProjectCustomField { id, .. }
        | UserProjectCustomField { id, .. }
        | VersionProjectCustomField { id, .. } => id.as_deref(),
    }
}

pub(super) fn project_custom_field_name(field: &models::ProjectCustomField) -> Option<String> {
    project_custom_field_def(field)
        .and_then(|custom| custom.name.as_ref())
        .map(|name| name.trim().to_string())
        .filter(|name| !name.is_empty())
}

pub(super) fn extract_custom_field_values_from_json(payload: &serde_json::Value) -> Vec<String> {
    let mut values = Vec::new();

    if let Some(bundle_values) = payload
        .get("bundle")
        .and_then(|bundle| bundle.get("values"))
        .and_then(serde_json::Value::as_array)
    {
        values.extend(bundle_values.iter().filter_map(extract_label_value));
    }

    if let Some(users) = payload
        .get("bundle")
        .and_then(|bundle| bundle.get("aggregatedUsers"))
        .and_then(serde_json::Value::as_array)
    {
        values.extend(users.iter().filter_map(extract_user_login));
    }

    if let Some(users) = payload
        .get("bundle")
        .and_then(|bundle| bundle.get("individuals"))
        .and_then(serde_json::Value::as_array)
    {
        values.extend(users.iter().filter_map(extract_user_login));
    }

    if let Some(defaults) = payload
        .get("defaultValues")
        .and_then(serde_json::Value::as_array)
    {
        values.extend(defaults.iter().filter_map(extract_label_or_login));
    }

    values
}

pub(super) fn normalize_values(values: Vec<String>) -> Vec<String> {
    let mut out: Vec<String> = values
        .into_iter()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .collect();
    out.sort();
    out.dedup();
    out
}

fn extract_label_or_login(value: &serde_json::Value) -> Option<String> {
    extract_label_value(value).or_else(|| extract_user_login(value))
}

fn extract_label_value(value: &serde_json::Value) -> Option<String> {
    value
        .get("localizedName")
        .and_then(serde_json::Value::as_str)
        .or_else(|| value.get("name").and_then(serde_json::Value::as_str))
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_string)
}

fn extract_user_login(value: &serde_json::Value) -> Option<String> {
    value
        .get("login")
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_string)
}

fn project_custom_field_def(field: &models::ProjectCustomField) -> Option<&models::CustomField> {
    use models::ProjectCustomField::*;

    match field {
        BuildProjectCustomField { field, .. }
        | BundleProjectCustomField { field, .. }
        | EnumProjectCustomField { field, .. }
        | GroupProjectCustomField { field, .. }
        | OwnedProjectCustomField { field, .. }
        | PeriodProjectCustomField { field, .. }
        | ProjectCustomField { field, .. }
        | SimpleProjectCustomField { field, .. }
        | StateProjectCustomField { field, .. }
        | TextProjectCustomField { field, .. }
        | UserProjectCustomField { field, .. }
        | VersionProjectCustomField { field, .. } => field.as_deref(),
    }
}
