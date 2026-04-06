use api::apis::default_api;
use api::models;

use crate::error::{Result, TrackItError};

use super::client::YouTrackClient;
use super::utils::map_api_error;

const ISSUE_LINKS_FIELDS: &str = "id,direction,linkType(name,sourceToTarget,targetToSource),issues(id,idReadable,summary),trimmedIssues(id,idReadable,summary)";

impl YouTrackClient {
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
}
