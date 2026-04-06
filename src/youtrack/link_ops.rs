use api::apis::default_api;
use api::models;

use crate::error::{Result, TrackItError};

use super::client::YouTrackClient;
use crate::utils::text::map_api_error;

const ISSUE_LINK_TYPES_FIELDS: &str = "id,name,sourceToTarget,targetToSource,directed";

impl YouTrackClient {
    pub async fn add_issue_link(
        &self,
        source_issue: &str,
        relation: &str,
        target_issue: &str,
    ) -> Result<()> {
        let source_link_id = self.resolve_link_id(source_issue, relation).await?;
        let target_id = self.resolve_issue_internal_id(target_issue).await?;

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
        let target_id = self.resolve_issue_internal_id(target_issue).await?;

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

    async fn resolve_link_id(&self, issue_ref: &str, relation: &str) -> Result<String> {
        let link_types = default_api::issue_link_types_get(
            &self.configuration,
            Some(ISSUE_LINK_TYPES_FIELDS),
            None,
            None,
        )
        .await
        .map_err(map_api_error)?;

        for link_type in link_types {
            let match_source_to_target = link_type
                .source_to_target
                .as_deref()
                .is_some_and(|name| name.eq_ignore_ascii_case(relation));
            let match_target_to_source = link_type
                .target_to_source
                .as_ref()
                .and_then(|name| name.as_deref())
                .is_some_and(|name| name.eq_ignore_ascii_case(relation));
            let match_type_name = link_type
                .name
                .as_deref()
                .is_some_and(|name| name.eq_ignore_ascii_case(relation));

            if match_source_to_target || match_target_to_source || match_type_name {
                let link_type_id = link_type.id.ok_or_else(|| {
                    TrackItError::ApiMessage(format!(
                        "Relation link type '{relation}' resolved without an id"
                    ))
                });

                let link_type_id = link_type_id?;
                let is_directed = link_type.directed.unwrap_or(false);

                if !is_directed {
                    return Ok(link_type_id);
                }

                if match_source_to_target {
                    return Ok(format!("{link_type_id}s"));
                }

                if match_target_to_source {
                    return Ok(format!("{link_type_id}t"));
                }

                return Err(TrackItError::Config(format!(
                    "Relation '{relation}' matches directed link type name only for issue '{issue_ref}'. Use its directional relation label."
                )));
            } else {
                continue;
            }
        }

        Err(TrackItError::Config(format!(
            "Unknown relation link '{relation}' for issue '{issue_ref}'"
        )))
    }
}
