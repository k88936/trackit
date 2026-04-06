use serde::Serialize;
use std::collections::BTreeSet;
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
    links: String,
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
            links: summarize_link_relations(issue),
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

fn summarize_link_relations(issue: &api::models::Issue) -> String {
    let Some(links) = &issue.links else {
        return String::new();
    };

    let mut relations: std::collections::BTreeMap<String, BTreeSet<String>> =
        std::collections::BTreeMap::new();
    for link in links {
        let relation = link_relation_label(link);
        let Some(relation) = relation else {
            continue;
        };

        let targets = relations.entry(relation).or_default();
        let related = link
            .issues
            .as_ref()
            .or(link.trimmed_issues.as_ref())
            .cloned()
            .unwrap_or_default();
        for linked in related {
            let ref_id = linked
                .id_readable
                .or(linked.id)
                .unwrap_or_else(String::new)
                .trim()
                .to_string();
            if !ref_id.is_empty() {
                targets.insert(ref_id);
            }
        }
    }

    relations
        .into_iter()
        .filter_map(|(relation, targets)| {
            if targets.is_empty() {
                None
            } else {
                Some(format!(
                    "{relation}:{}",
                    targets.into_iter().collect::<Vec<_>>().join(",")
                ))
            }
        })
        .collect::<Vec<_>>()
        .join("; ")
}

fn render_issue_links(issue: &api::models::Issue) {
    let mut rows: Vec<String> = Vec::new();
    let Some(links) = &issue.links else {
        return;
    };

    for link in links {
        let relation = link_relation_label(link).unwrap_or_default();
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

fn link_relation_label(link: &api::models::IssueLink) -> Option<String> {
    let link_type = link.link_type.as_ref()?;
    let source_to_target = link_type
        .source_to_target
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty());
    let target_to_source = link_type
        .target_to_source
        .as_ref()
        .and_then(|v| v.as_deref())
        .map(str::trim)
        .filter(|v| !v.is_empty());
    let name = link_type
        .name
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty());

    use api::models::issue_link::Direction;
    match link.direction {
        Some(Direction::Outward) => source_to_target
            .or(name)
            .map(str::to_string),
        Some(Direction::Inward) => target_to_source
            .or(name)
            .map(str::to_string),
        Some(Direction::Both) => match (source_to_target, target_to_source) {
            (Some(src), Some(tgt)) if src != tgt => Some(format!("{src} / {tgt}")),
            (Some(src), _) => Some(src.to_string()),
            (_, Some(tgt)) => Some(tgt.to_string()),
            _ => name.map(str::to_string),
        },
        None => source_to_target
            .or(target_to_source)
            .or(name)
            .map(str::to_string),
    }
}
