use api::apis::Error as ApiError;
use api::apis::configuration::Configuration;
use api::apis::default_api;
use api::models;

use crate::error::{Result, TrackItError};

const ME_FIELDS: &str = "id,login,fullName,email";
const PROJECT_FIELDS: &str = "id,name,shortName,archived";
const ISSUE_LIST_FIELDS: &str = "id,idReadable,summary,updated,project(id,name,shortName),customFields(name,value(name,fullName,login,idReadable))";
const ISSUE_DETAIL_FIELDS: &str = "id,idReadable,summary,description,created,updated,project(id,name,shortName),customFields(name,value(name,fullName,login,idReadable)),tags(name),links(id,direction,linkType(name,sourceToTarget,targetToSource),issues(id,idReadable,summary),trimmedIssues(id,idReadable,summary))";
const ISSUE_CREATE_FIELDS: &str = "id,idReadable,summary,description";
const ISSUE_COMMENT_FIELDS: &str = "id,text,author(id,login,fullName),created";
const ISSUE_LINKS_FIELDS: &str = "id,direction,linkType(name,sourceToTarget,targetToSource),issues(id,idReadable,summary),trimmedIssues(id,idReadable,summary)";

#[derive(Clone, Debug)]
pub struct ProjectFieldSuggestion {
    pub name: String,
    pub values: Vec<String>,
}

pub struct YouTrackClient {
    configuration: Configuration,
}

impl YouTrackClient {
    pub fn new(base_url: &str, token: &str) -> Result<Self> {
        let mut configuration = Configuration::new();
        configuration.base_path = normalize_base_path(base_url)?;
        configuration.bearer_access_token = Some(token.to_string());
        configuration.user_agent = Some(format!("trackit/{}", env!("CARGO_PKG_VERSION")));

        Ok(Self { configuration })
    }

    pub async fn me(&self) -> Result<models::Me> {
        default_api::users_me_get(&self.configuration, Some(ME_FIELDS))
            .await
            .map_err(map_api_error)
    }

    pub async fn list_projects(
        &self,
        skip: Option<i32>,
        top: Option<i32>,
    ) -> Result<Vec<models::Project>> {
        default_api::admin_projects_get(&self.configuration, Some(PROJECT_FIELDS), skip, top)
            .await
            .map_err(map_api_error)
    }

    pub async fn list_issues(
        &self,
        query: Option<&str>,
        skip: Option<i32>,
        top: Option<i32>,
    ) -> Result<Vec<models::Issue>> {
        default_api::issues_get(
            &self.configuration,
            query,
            None,
            Some(ISSUE_LIST_FIELDS),
            skip,
            top,
        )
        .await
        .map_err(map_api_error)
    }

    pub async fn get_issue(&self, id: &str) -> Result<models::Issue> {
        default_api::issues_id_get(&self.configuration, id, Some(ISSUE_DETAIL_FIELDS))
            .await
            .map_err(map_api_error)
    }

    pub async fn create_issue(
        &self,
        project_short_name: &str,
        summary: &str,
        description: Option<&str>,
    ) -> Result<models::Issue> {
        let mut issue = models::Issue::new();
        issue.summary = Some(Some(summary.to_string()));

        if let Some(description) = description {
            issue.description = Some(Some(description.to_string()));
        }

        let mut project = models::Project::new();
        project.short_name = Some(project_short_name.to_string());
        issue.project = Some(Box::new(project));

        default_api::issues_post(
            &self.configuration,
            None,
            None,
            Some(ISSUE_CREATE_FIELDS),
            Some(issue),
        )
        .await
        .map_err(map_api_error)
    }

    pub async fn comment_issue(&self, id: &str, text: &str) -> Result<models::IssueComment> {
        let mut comment = models::IssueComment::new();
        comment.text = Some(Some(text.to_string()));

        default_api::issues_id_comments_post(
            &self.configuration,
            id,
            None,
            None,
            Some(ISSUE_COMMENT_FIELDS),
            Some(comment),
        )
        .await
        .map_err(map_api_error)
    }

    pub async fn delete_issue(&self, id: &str) -> Result<()> {
        default_api::issues_id_delete(&self.configuration, id)
            .await
            .map_err(map_api_error)
    }

    pub async fn update_issue_field(&self, id: &str, key: &str, value: &str) -> Result<()> {
        let command = format!(
            "{} {}",
            quote_command_part(key),
            quote_command_part(value)
        );
        self.run_issue_command(id, &command).await
    }

    pub async fn list_project_values(&self) -> Result<Vec<String>> {
        let projects = default_api::admin_projects_get(
            &self.configuration,
            Some("name,shortName"),
            None,
            None,
        )
        .await
        .map_err(map_api_error)?;

        let mut values: Vec<String> = projects
            .into_iter()
            .filter_map(|p| {
                let short = p.short_name.unwrap_or_default();
                if short.is_empty() {
                    p.name
                } else {
                    Some(short)
                }
            })
            .collect();

        values.sort();
        values.dedup();
        Ok(values)
    }

    pub async fn list_project_custom_field_suggestions(
        &self,
        project: &str,
    ) -> Result<Vec<ProjectFieldSuggestion>> {
        let project_id = self.resolve_project_id(project).await?;
        let custom_fields = self.fetch_project_custom_fields(&project_id).await?;
        let mut suggestions = Vec::new();

        for field in custom_fields {
            let Some(name) = project_custom_field_name(&field) else {
                continue;
            };
            let Some(field_id) = project_custom_field_id(&field) else {
                continue;
            };
            let values = self
                .fetch_project_custom_field_value_labels(&project_id, field_id)
                .await
                .unwrap_or_default();
            suggestions.push(ProjectFieldSuggestion {
                name,
                values: normalize_values(values),
            });
        }

        suggestions.sort_by(|a, b| a.name.to_ascii_lowercase().cmp(&b.name.to_ascii_lowercase()));
        suggestions.dedup_by(|a, b| a.name.eq_ignore_ascii_case(&b.name));
        Ok(suggestions)
    }

    pub async fn get_issue_links(&self, id: &str) -> Result<Vec<models::IssueLink>> {
        default_api::issues_id_links_get(
            &self.configuration,
            id,
            Some(ISSUE_LINKS_FIELDS),
            None,
            None,
        )
        .await
        .map_err(map_api_error)
    }

    pub async fn add_issue_link(
        &self,
        source_issue: &str,
        relation: &str,
        target_issue: &str,
    ) -> Result<()> {
        let source_link_id = self.resolve_link_id(source_issue, relation).await?;
        let target_id = self.resolve_issue_id(target_issue).await?;

        let mut issue = models::Issue::new();
        issue.id = Some(target_id);

        default_api::issues_id_links_issue_link_id_issues_post(
            &self.configuration,
            source_issue,
            &source_link_id,
            None,
            Some("id,idReadable"),
            Some(issue),
        )
        .await
        .map_err(map_api_error)?;

        Ok(())
    }

    pub async fn remove_issue_link(
        &self,
        source_issue: &str,
        relation: &str,
        target_issue: &str,
    ) -> Result<()> {
        let source_link_id = self.resolve_link_id(source_issue, relation).await?;
        let target_id = self.resolve_issue_id(target_issue).await?;

        default_api::issues_id_links_issue_link_id_issues_issue_id_delete(
            &self.configuration,
            source_issue,
            &source_link_id,
            &target_id,
        )
        .await
        .map_err(map_api_error)?;

        Ok(())
    }

    async fn run_issue_command(&self, issue_id: &str, command: &str) -> Result<()> {
        let mut issue = models::Issue::new();
        issue.id = Some(issue_id.to_string());

        let mut payload = models::CommandList::new();
        payload.query = Some(Some(command.to_string()));
        payload.issues = Some(vec![issue]);

        default_api::commands_post(&self.configuration, None, None, Some(payload))
            .await
            .map_err(map_api_error)?;

        Ok(())
    }

    async fn resolve_issue_id(&self, issue_ref: &str) -> Result<String> {
        let issue = default_api::issues_id_get(&self.configuration, issue_ref, Some("id"))
            .await
            .map_err(map_api_error)?;

        issue.id.ok_or_else(|| {
            TrackItError::ApiMessage(format!(
                "Issue '{issue_ref}' resolved without an internal id"
            ))
        })
    }

    async fn resolve_link_id(&self, issue_ref: &str, relation: &str) -> Result<String> {
        let links = self.get_issue_links(issue_ref).await?;

        for link in links {
            let Some(link_type) = link.link_type.as_ref() else {
                continue;
            };

            let names = [
                link_type.name.as_deref(),
                link_type.source_to_target.as_deref(),
                link_type
                    .target_to_source
                    .as_ref()
                    .and_then(|v| v.as_ref().map(String::as_str)),
            ];

            let matched = names
                .into_iter()
                .flatten()
                .any(|name| name.eq_ignore_ascii_case(relation));
            if matched {
                if let Some(id) = link.id {
                    return Ok(id);
                }
            }
        }

        Err(TrackItError::Config(format!(
            "Unknown relation link '{relation}' for issue '{issue_ref}'"
        )))
    }

    async fn fetch_project_custom_fields(
        &self,
        project_id: &str,
    ) -> Result<Vec<models::ProjectCustomField>> {
        default_api::admin_projects_id_custom_fields_get(
            &self.configuration,
            project_id,
            Some("id,field(name,localizedName,aliases,fieldType(id,$type))"),
            None,
            None,
        )
        .await
        .map_err(map_api_error)
    }

    async fn fetch_project_custom_field_value_labels(
        &self,
        project_id: &str,
        project_custom_field_id: &str,
    ) -> Result<Vec<String>> {
        let endpoint = format!(
            "{}/admin/projects/{}/customFields/{}",
            self.configuration.base_path, project_id, project_custom_field_id
        );
        let fields = "bundle(values(name,localizedName),individuals(login,fullName),aggregatedUsers(login,fullName)),defaultValues(name,localizedName,login,fullName)";

        let mut request = self
            .configuration
            .client
            .get(endpoint)
            .query(&[("fields", fields)]);

        if let Some(user_agent) = self.configuration.user_agent.as_ref() {
            request = request.header("User-Agent", user_agent);
        }
        if let Some(token) = self.configuration.bearer_access_token.as_ref() {
            request = request.header("Authorization", format!("Bearer {token}"));
        }

        let response = request
            .send()
            .await
            .map_err(|err| TrackItError::ApiMessage(err.to_string()))?;

        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|err| TrackItError::ApiMessage(err.to_string()))?;

        if status.is_client_error() || status.is_server_error() {
            return Err(TrackItError::ApiResponse {
                status: status.as_u16(),
                body,
            });
        }

        let payload: serde_json::Value =
            serde_json::from_str(&body).map_err(|err| TrackItError::ApiMessage(err.to_string()))?;
        Ok(extract_custom_field_values_from_json(&payload))
    }

    async fn resolve_project_id(&self, project_key: &str) -> Result<String> {
        let projects = default_api::admin_projects_get(
            &self.configuration,
            Some("id,shortName,name"),
            None,
            None,
        )
        .await
        .map_err(map_api_error)?;

        let key = project_key.trim().to_ascii_lowercase();
        let project = projects.into_iter().find(|p| {
            p.id.as_deref()
                .map(|v| v.eq_ignore_ascii_case(&key))
                .unwrap_or(false)
                || p.short_name
                    .as_deref()
                    .map(|v| v.eq_ignore_ascii_case(&key))
                    .unwrap_or(false)
                || p.name
                    .as_deref()
                    .map(|v| v.eq_ignore_ascii_case(&key))
                    .unwrap_or(false)
        });

        let Some(project) = project else {
            return Err(TrackItError::Config(format!(
                "Unknown project '{project_key}'"
            )));
        };

        project.id.ok_or_else(|| {
            TrackItError::ApiMessage(format!(
                "Project '{project_key}' resolved without an internal id"
            ))
        })
    }
}

fn normalize_base_path(url: &str) -> Result<String> {
    let trimmed = url.trim().trim_end_matches('/');

    if !(trimmed.starts_with("http://") || trimmed.starts_with("https://")) {
        return Err(TrackItError::InvalidUrl(
            "URL must start with http:// or https://".to_string(),
        ));
    }

    if trimmed.ends_with("/api") {
        return Ok(trimmed.to_string());
    }

    Ok(format!("{trimmed}/api"))
}

fn map_api_error<T: std::fmt::Debug>(error: ApiError<T>) -> TrackItError {
    match error {
        ApiError::ResponseError(content) => TrackItError::ApiResponse {
            status: content.status.as_u16(),
            body: content.content,
        },
        other => TrackItError::ApiMessage(other.to_string()),
    }
}

fn project_custom_field_id(field: &models::ProjectCustomField) -> Option<&str> {
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

fn project_custom_field_name(field: &models::ProjectCustomField) -> Option<String> {
    project_custom_field_def(field)
        .and_then(|custom| custom.name.as_ref())
        .map(|name| name.trim().to_string())
        .filter(|name| !name.is_empty())
}

fn extract_custom_field_values_from_json(payload: &serde_json::Value) -> Vec<String> {
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

fn normalize_values(values: Vec<String>) -> Vec<String> {
    let mut out: Vec<String> = values
        .into_iter()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .collect();
    out.sort();
    out.dedup();
    out
}

fn quote_command_part(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.chars().any(|c| c.is_whitespace() || c == '"') {
        format!("\"{}\"", trimmed.replace('"', "\\\""))
    } else {
        trimmed.to_string()
    }
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
