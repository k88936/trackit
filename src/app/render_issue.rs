use serde::Serialize;
use tabled::Tabled;

use crate::error::Result;
use crate::output::{format_json, format_table};

use super::issue_fields::issue_custom_fields;
use super::render_basic::{opt_nested_str, opt_str};

#[derive(Serialize, Tabled)]
struct IssueRow {
    id_readable: String,
    summary: String,
    project: String,
    fields: String,
    updated: String,
}

pub fn render_issues(issues: &[api::models::Issue], as_json: bool) -> Result<()> {
    if as_json {
        println!("{}", format_json(&issues)?);
        return Ok(());
    }

    let rows: Vec<IssueRow> = issues
        .iter()
        .map(|issue| IssueRow {
            id_readable: opt_str(&issue.id_readable),
            summary: opt_nested_str(&issue.summary),
            project: issue
                .project
                .as_ref()
                .map(|p| {
                    let short = opt_str(&p.short_name);
                    if short.is_empty() {
                        opt_str(&p.name)
                    } else {
                        short
                    }
                })
                .unwrap_or_default(),
            fields: summarize_issue_fields(issue),
            updated: issue.updated.map(|t| t.to_string()).unwrap_or_default(),
        })
        .collect();

    println!("{}", format_table(&rows));
    Ok(())
}

pub fn render_issue_detail(issue: &api::models::Issue, as_json: bool) -> Result<()> {
    if as_json {
        println!("{}", format_json(issue)?);
        return Ok(());
    }

    println!("id: {}", opt_str(&issue.id));
    println!("id_readable: {}", opt_str(&issue.id_readable));
    println!("summary: {}", opt_nested_str(&issue.summary));
    println!("description: {}", opt_nested_str(&issue.description));
    println!(
        "created: {}",
        issue.created.map(|t| t.to_string()).unwrap_or_default()
    );
    println!(
        "updated: {}",
        issue.updated.map(|t| t.to_string()).unwrap_or_default()
    );

    if let Some(project) = &issue.project {
        println!(
            "project: {} ({})",
            opt_str(&project.name),
            opt_str(&project.short_name)
        );
    }
    let fields = issue_custom_fields(issue);
    if fields.is_empty() {
        println!("fields: (none)");
    } else {
        println!("fields:");
        for (name, value) in fields {
            if value.is_empty() {
                println!("  - {name}:");
            } else {
                println!("  - {name}: {value}");
            }
        }
    }
    if let Some(tags) = &issue.tags {
        let names: Vec<String> = tags.iter().map(|tag| opt_str(&tag.name)).collect();
        println!("tags: {}", names.join(", "));
    }
    render_issue_links(issue);

    Ok(())
}

fn summarize_issue_fields(issue: &api::models::Issue) -> String {
    let pairs = issue_custom_fields(issue);
    if pairs.is_empty() {
        return String::new();
    }

    let summary = pairs
        .into_iter()
        .map(|(name, value)| {
            if value.is_empty() {
                name
            } else {
                format!("{name}={value}")
            }
        })
        .collect::<Vec<_>>()
        .join("; ");

    const LIMIT: usize = 80;
    if summary.chars().count() > LIMIT {
        let mut out = String::new();
        for ch in summary.chars().take(LIMIT - 3) {
            out.push(ch);
        }
        out.push_str("...");
        out
    } else {
        summary
    }
}

fn render_issue_links(issue: &api::models::Issue) {
    let mut rows: Vec<String> = Vec::new();
    let Some(links) = &issue.links else {
        return;
    };

    for link in links {
        let relation = link
            .link_type
            .as_ref()
            .and_then(|lt| lt.name.clone())
            .unwrap_or_default();
        let related = link
            .issues
            .as_ref()
            .or(link.trimmed_issues.as_ref())
            .cloned()
            .unwrap_or_default();

        for linked in related {
            let id_readable = linked.id_readable.or(linked.id).unwrap_or_else(String::new);
            let summary = linked.summary.flatten().unwrap_or_default();
            rows.push(format!("{relation}: {id_readable} {summary}"));
        }
    }

    if !rows.is_empty() {
        println!("links:");
        for row in rows {
            println!("  - {row}");
        }
    }
}
