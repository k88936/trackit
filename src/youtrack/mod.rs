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

    pub async fn add_tag(&self, id: &str, tag_name: &str) -> Result<models::Tag> {
        let mut tag = models::Tag::new();
        tag.name = Some(tag_name.to_string());

        default_api::issues_id_tags_post(&self.configuration, id, Some("id,name"), Some(tag))
            .await
            .map_err(map_api_error)
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
