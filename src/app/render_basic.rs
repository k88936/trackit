use serde::Serialize;
use tabled::Tabled;

use crate::error::Result;
use crate::output::{format_json, format_table};
use crate::youtrack::ProjectFieldSuggestion;

#[derive(Serialize, Tabled)]
struct MeRow {
    id: String,
    login: String,
    full_name: String,
    email: String,
}

pub fn render_me(me: &api::models::Me, as_json: bool) -> Result<()> {
    if as_json {
        println!("{}", format_json(me)?);
        return Ok(());
    }

    let row = MeRow {
        id: opt_str(&me.id),
        login: opt_str(&me.login),
        full_name: opt_str(&me.full_name),
        email: opt_nested_str(&me.email),
    };

    println!("{}", format_table(&[row]));
    Ok(())
}

#[derive(Serialize, Tabled)]
struct ProjectRow {
    id: String,
    short_name: String,
    name: String,
    archived: bool,
}

pub fn render_projects(projects: &[api::models::Project], as_json: bool) -> Result<()> {
    if as_json {
        println!("{}", format_json(&projects)?);
        return Ok(());
    }

    let rows: Vec<ProjectRow> = projects
        .iter()
        .map(|p| ProjectRow {
            id: opt_str(&p.id),
            short_name: opt_str(&p.short_name),
            name: opt_str(&p.name),
            archived: p.archived.unwrap_or(false),
        })
        .collect();

    println!("{}", format_table(&rows));
    Ok(())
}

#[derive(Serialize, Tabled)]
struct ProjectCustomFieldRow {
    name: String,
    values: String,
}

pub fn render_project_custom_fields(
    fields: &[ProjectFieldSuggestion],
    as_json: bool,
    summarize_values: impl Fn(&[String]) -> String,
) -> Result<()> {
    if as_json {
        println!("{}", format_json(fields)?);
        return Ok(());
    }

    let rows: Vec<ProjectCustomFieldRow> = fields
        .iter()
        .map(|field| ProjectCustomFieldRow {
            name: field.name.clone(),
            values: summarize_values(&field.values),
        })
        .collect();

    println!("{}", format_table(&rows));
    Ok(())
}

#[derive(Serialize, Tabled)]
struct CommentRow {
    id: String,
    text: String,
    created: String,
}

pub fn render_comment(comment: &api::models::IssueComment, as_json: bool) -> Result<()> {
    if as_json {
        println!("{}", format_json(comment)?);
        return Ok(());
    }

    let row = CommentRow {
        id: opt_str(&comment.id),
        text: opt_nested_str(&comment.text),
        created: comment.created.map(|t| t.to_string()).unwrap_or_default(),
    };

    println!("{}", format_table(&[row]));
    Ok(())
}

pub(crate) fn opt_str(value: &Option<String>) -> String {
    value.clone().unwrap_or_default()
}

pub(crate) fn opt_nested_str(value: &Option<Option<String>>) -> String {
    value.clone().flatten().unwrap_or_default()
}
