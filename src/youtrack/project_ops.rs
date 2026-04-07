use api::apis::default_api;
use api::models;

use crate::error::{Result, TrackItError};

use super::client::YouTrackClient;
use super::project_field_helpers::{
    ProjectFieldSuggestion, extract_custom_field_values_from_json, normalize_values,
    project_custom_field_id, project_custom_field_name,
};
use crate::utils::text::map_api_error;

const PROJECT_FIELDS: &str = "id,name,shortName,archived";
const PROJECT_DETAIL_FIELDS: &str =
    "id,name,shortName,archived,description,leader(id,login,fullName),team(id,name)";

#[derive(Clone, Debug, serde::Serialize)]
pub struct ProjectDetail {
    pub project: models::Project,
    pub custom_fields: Vec<ProjectFieldSuggestion>,
}

impl YouTrackClient {
    pub async fn list_projects(
        &self,
        skip: Option<i32>,
        top: Option<i32>,
    ) -> Result<Vec<models::Project>> {
        default_api::admin_projects_get(&self.configuration, Some(PROJECT_FIELDS), skip, top)
            .await
            .map_err(map_api_error)
    }

    pub async fn get_project_detail(&self, project: &str) -> Result<ProjectDetail> {
        let project_id = self.resolve_project_id(project).await?;
        let project =
            default_api::admin_projects_id_get(&self.configuration, &project_id, Some(PROJECT_DETAIL_FIELDS))
                .await
                .map_err(map_api_error)?;
        let custom_fields = self
            .list_project_custom_field_suggestions_by_id(&project_id)
            .await?;
        Ok(ProjectDetail {
            project,
            custom_fields,
        })
    }

    async fn list_project_custom_field_suggestions_by_id(
        &self,
        project_id: &str,
    ) -> Result<Vec<ProjectFieldSuggestion>> {
        let custom_fields = self.fetch_project_custom_fields(project_id).await?;
        let mut suggestions = Vec::new();

        for field in custom_fields {
            let Some(name) = project_custom_field_name(&field) else {
                continue;
            };
            let Some(field_id) = project_custom_field_id(&field) else {
                continue;
            };
            let values = self
                .fetch_project_custom_field_value_labels(project_id, field_id)
                .await
                .unwrap_or_default();
            suggestions.push(ProjectFieldSuggestion {
                name,
                values: normalize_values(values),
            });
        }

        suggestions.sort_by(|a, b| {
            a.name
                .to_ascii_lowercase()
                .cmp(&b.name.to_ascii_lowercase())
        });
        suggestions.dedup_by(|a, b| a.name.eq_ignore_ascii_case(&b.name));
        Ok(suggestions)
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
