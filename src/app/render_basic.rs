use serde::Serialize;
use tabled::Tabled;

use crate::error::Result;
use crate::output::{format_json, format_table};
use crate::youtrack::ProjectDetail;

#[derive(Serialize, Tabled)]
struct MeRow {
    login: String,
    full_name: String,
}

pub fn render_me(me: &api::models::Me, as_json: bool) -> Result<()> {
    if as_json {
        println!("{}", format_json(me)?);
        return Ok(());
    }

    let row = MeRow {
        login: opt_str(&me.login),
        full_name: opt_str(&me.full_name),
    };

    println!("{}", format_table(&[row]));
    Ok(())
}

#[derive(Serialize, Tabled)]
struct ProjectRow {
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
            short_name: opt_str(&p.short_name),
            name: opt_str(&p.name),
            archived: p.archived.unwrap_or(false),
        })
        .collect();

    println!("{}", format_table(&rows));
    Ok(())
}

pub fn render_project_detail(
    detail: &ProjectDetail,
    as_json: bool,
    summarize_values: impl Fn(&[String]) -> String,
) -> Result<()> {
    if as_json {
        println!("{}", format_json(detail)?);
        return Ok(());
    }

    let project = &detail.project;
    println!("short_name: {}", opt_str(&project.short_name));
    println!("name: {}", opt_str(&project.name));
    println!("archived: {}", project.archived.unwrap_or(false));
    println!("description: {}", opt_nested_str(&project.description));
    println!(
        "leader: {}",
        project
            .leader
            .as_ref()
            .map(|u| user_display_name(u.as_ref()))
            .unwrap_or_default()
    );

    if let Some(team) = &project.team {
        println!("team: {}", opt_str(&team.name));
    }

    if detail.custom_fields.is_empty() {
        println!("custom_fields: (none)");
    } else {
        println!("custom_fields:");
        for field in &detail.custom_fields {
            println!("  - {}: {}", field.name, summarize_values(&field.values));
        }
    }

    Ok(())
}

#[derive(Serialize, Tabled)]
struct CommentRow {
    text: String,
}

pub fn render_comment(comment: &api::models::IssueComment, as_json: bool) -> Result<()> {
    if as_json {
        println!("{}", format_json(comment)?);
        return Ok(());
    }

    let row = CommentRow {
        text: opt_nested_str(&comment.text),
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

fn user_display_name(user: &api::models::User) -> String {
    use api::models::User;

    let (full_name, login, id) = match user {
        User::Me {
            full_name,
            login,
            id,
            ..
        }
        | User::User {
            full_name,
            login,
            id,
            ..
        }
        | User::VcsUnresolvedUser {
            full_name,
            login,
            id,
            ..
        } => (full_name, login, id),
    };

    full_name
        .clone()
        .or_else(|| login.clone())
        .or_else(|| id.clone())
        .unwrap_or_default()
}
