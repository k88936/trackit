use api::apis::default_api;
use api::models;
use std::collections::BTreeSet;

use crate::error::Result;

use super::client::YouTrackClient;
use super::project_field_helpers::project_custom_field_type_id;
use super::utils::{map_api_error, quote_command_part};

const ME_FIELDS: &str = "id,login,fullName,email";
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
        project: Option<&str>,
        skip: Option<i32>,
        top: Option<i32>,
    ) -> Result<Vec<models::Issue>> {
        let fields = self.build_issue_fields_mask(IssueFieldMaskKind::List, project).await;
        default_api::issues_get(
            &self.configuration,
            query,
            None,
            Some(fields.as_str()),
            skip,
            top,
        )
        .await
        .map_err(map_api_error)
    }

    pub async fn get_issue(&self, id: &str) -> Result<models::Issue> {
        let project = self.get_issue_project_key(id).await?;
        let fields = self
            .build_issue_fields_mask(IssueFieldMaskKind::Detail, project.as_deref())
            .await;

        default_api::issues_id_get(&self.configuration, id, Some(fields.as_str()))
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

        let fields = self
            .build_issue_fields_mask(IssueFieldMaskKind::Create, Some(project_short_name))
            .await;

        default_api::issues_post(
            &self.configuration,
            None,
            None,
            Some(fields.as_str()),
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

    async fn get_issue_project_key(&self, id: &str) -> Result<Option<String>> {
        let issue = default_api::issues_id_get(&self.configuration, id, Some("project(id,shortName,name)"))
            .await
            .map_err(map_api_error)?;

        let project = issue.project.as_deref();
        let key = project
            .and_then(|p| p.short_name.clone())
            .or_else(|| project.and_then(|p| p.id.clone()))
            .or_else(|| project.and_then(|p| p.name.clone()))
            .map(|v| v.trim().to_string())
            .filter(|v| !v.is_empty());
        Ok(key)
    }

    async fn build_issue_fields_mask(
        &self,
        kind: IssueFieldMaskKind,
        project: Option<&str>,
    ) -> String {
        let custom_value_mask = self
            .build_custom_field_value_mask(project)
            .await
            .unwrap_or_else(|_| default_custom_field_value_mask());

        let mut parts = Vec::new();
        parts.extend([
            "id".to_string(),
            "idReadable".to_string(),
            "summary".to_string(),
            "project(id,name,shortName)".to_string(),
            format!("customFields(name,value({custom_value_mask}))"),
        ]);

        match kind {
            IssueFieldMaskKind::List => {
                parts.push("updated".to_string());
                parts.push(
                    "links(linkType(name),issues(id,idReadable),trimmedIssues(id,idReadable))"
                        .to_string(),
                );
            }
            IssueFieldMaskKind::Detail => {
                parts.extend([
                    "description".to_string(),
                    "created".to_string(),
                    "updated".to_string(),
                    "tags(name)".to_string(),
                    "links(id,direction,linkType(name,sourceToTarget,targetToSource),issues(id,idReadable,summary),trimmedIssues(id,idReadable,summary))".to_string(),
                ]);
            }
            IssueFieldMaskKind::Create => {
                parts.push("description".to_string());
            }
        }

        parts.join(",")
    }

    async fn build_custom_field_value_mask(&self, project: Option<&str>) -> Result<String> {
        let Some(project) = project.map(str::trim).filter(|v| !v.is_empty()) else {
            return Ok(default_custom_field_value_mask());
        };

        let project_id = self.resolve_project_id_for_issue_fields(project).await?;
        let custom_fields = default_api::admin_projects_id_custom_fields_get(
            &self.configuration,
            &project_id,
            Some("field(fieldType(id))"),
            None,
            None,
        )
        .await
        .map_err(map_api_error)?;

        let mut value_fields = BTreeSet::new();
        value_fields.insert("id".to_string());

        for field in custom_fields {
            let field_type_id = project_custom_field_type_id(&field)
                .map(|v| v.to_ascii_lowercase())
                .unwrap_or_default();
            for value_field in value_fields_for_type(&field_type_id) {
                value_fields.insert((*value_field).to_string());
            }
        }

        Ok(value_fields.into_iter().collect::<Vec<_>>().join(","))
    }

    async fn resolve_project_id_for_issue_fields(&self, project_key: &str) -> Result<String> {
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

        project.and_then(|p| p.id).ok_or_else(|| {
            crate::error::TrackItError::Config(format!("Unknown project '{project_key}'"))
        })
    }
}

#[derive(Clone, Copy)]
enum IssueFieldMaskKind {
    List,
    Detail,
    Create,
}

fn default_custom_field_value_mask() -> String {
    "id,name,localizedName,fullName,login,idReadable,presentation,text,minutes".to_string()
}

fn value_fields_for_type(field_type_id: &str) -> &'static [&'static str] {
    if field_type_id.contains("user") || field_type_id.contains("owned") {
        return &["name", "fullName", "login", "idReadable"];
    }
    if field_type_id.contains("group") {
        return &["name"];
    }
    if field_type_id.contains("period") {
        return &["minutes", "presentation"];
    }
    if field_type_id.contains("text") {
        return &["text"];
    }
    if field_type_id.contains("enum")
        || field_type_id.contains("state")
        || field_type_id.contains("build")
        || field_type_id.contains("version")
    {
        return &["name", "localizedName"];
    }

    &["name"]
}
