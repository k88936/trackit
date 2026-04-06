use api::apis::default_api;
use api::models;

use crate::error::Result;

use super::client::YouTrackClient;
use super::utils::{map_api_error, quote_command_part};

const ME_FIELDS: &str = "id,login,fullName,email";
const ISSUE_LIST_FIELDS: &str = "id,idReadable,summary,updated,project(id,name,shortName),customFields(name,value(name,fullName,login,idReadable))";
const ISSUE_DETAIL_FIELDS: &str = "id,idReadable,summary,description,created,updated,project(id,name,shortName),customFields(name,value(name,fullName,login,idReadable)),tags(name),links(id,direction,linkType(name,sourceToTarget,targetToSource),issues(id,idReadable,summary),trimmedIssues(id,idReadable,summary))";
const ISSUE_CREATE_FIELDS: &str = "id,idReadable,summary,description";
const ISSUE_COMMENT_FIELDS: &str = "id,text,author(id,login,fullName),created";

impl YouTrackClient {
    pub async fn me(&self) -> Result<models::Me> {
        default_api::users_me_get(&self.configuration, Some(ME_FIELDS))
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
        let command = format!("{} {}", quote_command_part(key), quote_command_part(value));
        self.run_issue_command(id, &command).await
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
