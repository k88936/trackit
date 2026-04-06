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

    pub async fn assign_issue(&self, id: &str, user: &str) -> Result<()> {
        self.run_issue_command(id, &format!("Assignee {}", user))
            .await
    }

    pub async fn update_issue_state(&self, id: &str, state: &str) -> Result<()> {
        self.run_issue_command(id, &format!("State {}", state))
            .await
    }

    pub async fn update_issue_project(&self, id: &str, project: &str) -> Result<()> {
        self.run_issue_command(id, &format!("project {}", project))
            .await
    }

    pub async fn update_issue_priority(&self, id: &str, priority: &str) -> Result<()> {
        self.run_issue_command(id, &format!("Priority {}", priority))
            .await
    }

    pub async fn update_issue_type(&self, id: &str, issue_type: &str) -> Result<()> {
        self.run_issue_command(id, &format!("Type {}", issue_type))
            .await
    }

    pub async fn update_issue_text(
        &self,
        id: &str,
        summary: Option<&str>,
        description: Option<&str>,
    ) -> Result<models::Issue> {
        let mut issue = models::Issue::new();

        if let Some(summary) = summary {
            issue.summary = Some(Some(summary.to_string()));
        }

        if let Some(description) = description {
            issue.description = Some(Some(description.to_string()));
        }

        default_api::issues_id_post(
            &self.configuration,
            id,
            None,
            Some(ISSUE_DETAIL_FIELDS),
            Some(issue),
        )
        .await
        .map_err(map_api_error)
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

    pub async fn suggest_list_values_for_project(
        &self,
        project: &str,
        field: &str,
    ) -> Result<Vec<String>> {
        let Some(project_field) = self.resolve_project_field_name(project, field).await? else {
            return Ok(Vec::new());
        };
        let query = format!(
            "project:{} {}: ",
            quote_search_query_value(project),
            quote_search_field_name(&project_field)
        );
        let mut payload = models::SearchSuggestions::new();
        payload.query = Some(Some(query.clone()));
        payload.caret = Some(query.len() as i32);

        let response = default_api::search_assist_post(
            &self.configuration,
            Some("suggestions(option,description)"),
            Some(payload),
        )
        .await
        .map_err(map_api_error)?;

        Ok(extract_suggestion_values(response.suggestions))
    }

    pub async fn suggest_update_values_for_project(
        &self,
        project: &str,
        field: &str,
    ) -> Result<Vec<String>> {
        let Some(project_field) = self.resolve_project_field_name(project, field).await? else {
            return Ok(Vec::new());
        };
        let query = format!(
            "project {} {} ",
            quote_command_value(project),
            quote_command_field_name(&project_field)
        );
        let mut payload = models::CommandList::new();
        payload.query = Some(Some(query.clone()));
        payload.caret = Some(query.len() as i32);

        let response = default_api::commands_assist_post(
            &self.configuration,
            Some("suggestions(option,description)"),
            Some(payload),
        )
        .await
        .map_err(map_api_error)?;

        Ok(extract_suggestion_values(response.suggestions))
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

    async fn resolve_project_field_name(
        &self,
        project_key: &str,
        logical_field: &str,
    ) -> Result<Option<String>> {
        let project_id = self.resolve_project_id(project_key).await?;
        let custom_fields = default_api::admin_projects_id_custom_fields_get(
            &self.configuration,
            &project_id,
            Some("field(name,localizedName,aliases,fieldType(id,$type))"),
            None,
            None,
        )
        .await
        .map_err(map_api_error)?;

        Ok(select_project_field_name(&custom_fields, logical_field))
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

fn extract_suggestion_values(suggestions: Option<Vec<models::Suggestion>>) -> Vec<String> {
    let mut values: Vec<String> = suggestions
        .unwrap_or_default()
        .into_iter()
        .filter_map(|s| {
            let option = s.option.and_then(|v| v);
            let description = s.description.and_then(|v| v);
            option.or(description)
        })
        .filter(|v| !v.trim().is_empty())
        .collect();

    values.sort();
    values.dedup();
    values
}

fn quote_search_query_value(value: &str) -> String {
    if value.chars().any(char::is_whitespace) {
        format!("\"{}\"", value.replace('"', "\\\""))
    } else {
        value.to_string()
    }
}

fn quote_command_value(value: &str) -> String {
    if value.chars().any(char::is_whitespace) {
        format!("\"{}\"", value.replace('"', "\\\""))
    } else {
        value.to_string()
    }
}

fn quote_search_field_name(value: &str) -> String {
    if value.chars().any(char::is_whitespace) {
        format!("\"{}\"", value.replace('"', "\\\""))
    } else {
        value.to_string()
    }
}

fn quote_command_field_name(value: &str) -> String {
    if value.chars().any(char::is_whitespace) {
        format!("\"{}\"", value.replace('"', "\\\""))
    } else {
        value.to_string()
    }
}

fn select_project_field_name(
    fields: &[models::ProjectCustomField],
    logical_field: &str,
) -> Option<String> {
    let mut best_score = 0i32;
    let mut best_name: Option<String> = None;

    for field in fields {
        let Some(custom) = project_custom_field_def(field) else {
            continue;
        };
        let Some(name) = custom.name.as_deref() else {
            continue;
        };

        let score = score_project_field(custom, logical_field);
        if score > best_score {
            best_score = score;
            best_name = Some(name.to_string());
        }
    }

    best_name
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

fn score_project_field(field: &models::CustomField, logical_field: &str) -> i32 {
    let name = field
        .name
        .as_deref()
        .unwrap_or_default()
        .to_ascii_lowercase();
    let localized = field
        .localized_name
        .as_ref()
        .and_then(|v| v.as_ref())
        .map(|v| v.to_ascii_lowercase())
        .unwrap_or_default();
    let aliases = field
        .aliases
        .as_ref()
        .and_then(|v| v.as_ref())
        .map(|v| v.to_ascii_lowercase())
        .unwrap_or_default();
    let field_type = field
        .field_type
        .as_ref()
        .and_then(|t| t.dollar_type.as_ref())
        .map(|v| v.to_ascii_lowercase())
        .unwrap_or_default();

    let has = |needle: &str| {
        name.contains(needle) || localized.contains(needle) || aliases.contains(needle)
    };

    let key = logical_field.trim().to_ascii_lowercase();
    match key.as_str() {
        "state" => {
            if field_type.contains("statefieldtype") {
                return 100;
            }
            if has("state") {
                return 80;
            }
            0
        }
        "assignee" => {
            if has("assignee") {
                return 100;
            }
            if has("assign") && field_type.contains("userfieldtype") {
                return 90;
            }
            if field_type.contains("userfieldtype") {
                return 40;
            }
            0
        }
        "priority" => {
            if has("priority") {
                return 100;
            }
            if has("severity") {
                return 80;
            }
            0
        }
        "type" => {
            if has("issue type") {
                return 100;
            }
            if has("type") {
                return 80;
            }
            if field_type.contains("enumfieldtype") && !has("priority") {
                return 30;
            }
            0
        }
        _ => 0,
    }
}
