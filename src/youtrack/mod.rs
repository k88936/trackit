use api::apis::Error as ApiError;
use api::apis::configuration::Configuration;
use api::apis::default_api;
use api::models;

use crate::error::{Result, TrackItError};

const ME_FIELDS: &str = "id,login,fullName,email";
const PROJECT_FIELDS: &str = "id,name,shortName,archived";
const ISSUE_LIST_FIELDS: &str = "id,idReadable,summary,updated,project(id,name,shortName),customFields(name,value(name,fullName,login,idReadable))";
const ISSUE_DETAIL_FIELDS: &str = "id,idReadable,summary,description,created,updated,project(id,name,shortName),customFields(name,value(name,fullName,login,idReadable)),tags(name)";
const ISSUE_CREATE_FIELDS: &str = "id,idReadable,summary,description";
const ISSUE_COMMENT_FIELDS: &str = "id,text,author(id,login,fullName),created";

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

    pub async fn suggest_list_values(&self, field: &str) -> Result<Vec<String>> {
        let query = format!("{field}: ");
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

    pub async fn suggest_update_values(&self, field: &str) -> Result<Vec<String>> {
        let query = format!("{field} ");
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
