mod cli;
mod config;
mod error;
mod output;
mod youtrack;

use clap::{Args, CommandFactory, Parser, Subcommand};
use cli::run_setup_wizard;
use serde::Serialize;
use std::env;
use tabled::Tabled;

use crate::config::Config;
use crate::error::{Result, TrackItError};
use crate::output::{format_json, format_table};
use crate::youtrack::{ProjectFieldSuggestion, YouTrackClient};

#[derive(Parser)]
#[command(name = "trackit")]
#[command(
    about = "YouTrack CLI tool",
    long_about = "Youtrack CLI Tool , especially for code agent to do issue management"
)]
#[command(version)]
struct Cli {
    #[arg(long, global = true, help = "Output in JSON format")]
    json: bool,

    #[arg(
        long,
        global = true,
        help = "Override YouTrack URL",
        env = "YOUTRACK_URL"
    )]
    url: Option<String>,

    #[arg(
        long,
        global = true,
        help = "Override API token",
        env = "YOUTRACK_TOKEN"
    )]
    token: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Clone)]
struct GlobalOpts {
    json: bool,
    url: Option<String>,
    token: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Interactive setup wizard")]
    SetupWizard,

    #[command(about = "Show current authenticated user")]
    Me,

    #[command(about = "Project operations")]
    Projects {
        #[command(subcommand)]
        command: ProjectCommands,
    },

    #[command(about = "Issue operations")]
    Issues {
        #[command(subcommand)]
        command: IssueCommands,
    },
}

#[derive(Subcommand)]
enum ProjectCommands {
    #[command(about = "List projects")]
    List {
        #[arg(long)]
        skip: Option<i32>,
        #[arg(long)]
        top: Option<i32>,
    },
}

#[derive(Subcommand)]
enum IssueCommands {
    #[command(about = "List issues")]
    List {
        #[arg(long)]
        project: String,
        #[arg(long = "filter", value_name = "KEY=VALUE")]
        filters: Vec<String>,
        #[arg(long)]
        skip: Option<i32>,
        #[arg(long)]
        top: Option<i32>,
    },

    #[command(about = "Read issue details")]
    Read { id: String },

    #[command(about = "Create issue")]
    Create(IssueCreateArgs),

    #[command(about = "Delete issue")]
    Delete { id: String },

    #[command(about = "Add comment to issue")]
    Comment {
        id: String,
        #[arg(long)]
        text: String,
    },

    #[command(about = "Update issue fields")]
    Update(IssueUpdateArgs),
}

#[derive(Args)]
struct IssueCreateArgs {
    #[arg(long)]
    project: String,
    #[arg(long)]
    summary: String,
    #[arg(long)]
    description: Option<String>,
    #[arg(long = "field", value_name = "KEY=VALUE")]
    fields: Vec<String>,
    #[arg(long = "link", value_name = "RELATION:ISSUE")]
    links: Vec<String>,
}

#[derive(Args)]
struct IssueUpdateArgs {
    id: String,
    #[arg(long)]
    project: String,
    #[arg(long = "field", value_name = "KEY=VALUE")]
    field: Vec<String>,
    #[arg(long = "link", value_name = "RELATION:ISSUE")]
    link: Vec<String>,
    #[arg(long = "unlink", value_name = "RELATION:ISSUE")]
    unlink: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    if try_print_dynamic_help().await? {
        return Ok(());
    }

    let cli = Cli::parse();
    let global = GlobalOpts {
        json: cli.json,
        url: cli.url.clone(),
        token: cli.token.clone(),
    };

    match cli.command {
        Commands::SetupWizard => run_setup_wizard().await?,
        Commands::Me => {
            let client = build_client(&global)?;
            let me = client.me().await?;
            render_me(&me, global.json)?;
        }
        Commands::Projects { command } => {
            let client = build_client(&global)?;
            match command {
                ProjectCommands::List { skip, top } => {
                    let projects = client.list_projects(skip, top).await?;
                    render_projects(&projects, global.json)?;
                }
            }
        }
        Commands::Issues { command } => {
            let client = build_client(&global)?;
            match command {
                IssueCommands::List {
                    project,
                    filters,
                    skip,
                    top,
                } => {
                    let parsed_filters = parse_key_value_specs(&filters, "--filter")?;
                    let query = build_issue_query(&project, &parsed_filters);
                    let issues = client.list_issues(Some(&query), skip, top).await?;
                    render_issues(&issues, global.json)?;
                }
                IssueCommands::Read { id } => {
                    let issue = client.get_issue(&id).await?;
                    render_issue_detail(&issue, global.json)?;
                }
                IssueCommands::Create(args) => {
                    let issue = client
                        .create_issue(&args.project, &args.summary, args.description.as_deref())
                        .await?;
                    let issue_ref =
                        issue
                            .id_readable
                            .clone()
                            .or(issue.id.clone())
                            .ok_or_else(|| {
                                TrackItError::ApiMessage(
                                    "Created issue response is missing id and idReadable"
                                        .to_string(),
                                )
                            })?;

                    for link in &args.links {
                        let (relation, target) = parse_link_spec(link)?;
                        client
                            .add_issue_link(&issue_ref, &relation, &target)
                            .await?;
                    }

                    let issue = client.get_issue(&issue_ref).await?;
                    render_issue_detail(&issue, global.json)?;
                }
                IssueCommands::Delete { id } => {
                    client.delete_issue(&id).await?;
                    println!("Deleted issue {id}");
                }
                IssueCommands::Comment { id, text } => {
                    let comment = client.comment_issue(&id, &text).await?;
                    render_comment(&comment, global.json)?;
                }
                IssueCommands::Update(args) => {
                    let assignments = parse_key_value_specs(&args.field, "--set")?;
                    for (key, value) in assignments {
                        client.update_issue_field(&args.id, &key, &value).await?;
                    }

                    for link in &args.link {
                        let (relation, target) = parse_link_spec(link)?;
                        client.add_issue_link(&args.id, &relation, &target).await?;
                    }

                    for link in &args.unlink {
                        let (relation, target) = parse_link_spec(link)?;
                        client
                            .remove_issue_link(&args.id, &relation, &target)
                            .await?;
                    }

                    let issue = client.get_issue(&args.id).await?;
                    render_issue_detail(&issue, global.json)?;
                }
            }
        }
    }

    Ok(())
}

async fn try_print_dynamic_help() -> Result<bool> {
    let args: Vec<String> = env::args().collect();
    if !args.iter().any(|arg| arg == "--help" || arg == "-h") {
        return Ok(false);
    }

    let target = if args.iter().any(|arg| arg == "issues") && args.iter().any(|arg| arg == "list") {
        Some(("issues", "list"))
    } else if args.iter().any(|arg| arg == "issues") && args.iter().any(|arg| arg == "update") {
        Some(("issues", "update"))
    } else if args.iter().any(|arg| arg == "issues") && args.iter().any(|arg| arg == "create") {
        Some(("issues", "create"))
    } else {
        None
    };

    let Some((top, sub)) = target else {
        return Ok(false);
    };

    let global = parse_global_opts_from_args(&args);
    let mut root = Cli::command();
    if let Some(command) = root
        .find_subcommand_mut(top)
        .and_then(|c| c.find_subcommand_mut(sub))
    {
        let project = parse_subcommand_project_from_args(&args);
        let after_help = match build_client(&global) {
            Ok(client) => build_dynamic_after_help(&client, sub, project.as_deref()).await,
            Err(err) => format!(
                "Dynamic values unavailable: {err}\nProvide `--url` and `--token` (or env/config), then run help again."
            ),
        };
        let mut command_with_help = command.clone().after_help(after_help);
        command_with_help.print_long_help()?;
        println!();
        return Ok(true);
    }

    Ok(false)
}

fn parse_global_opts_from_args(args: &[String]) -> GlobalOpts {
    let mut url = None;
    let mut token = None;

    let mut i = 1usize;
    while i < args.len() {
        let arg = &args[i];
        if let Some(value) = arg.strip_prefix("--url=") {
            url = Some(value.to_string());
            i += 1;
            continue;
        }
        if let Some(value) = arg.strip_prefix("--token=") {
            token = Some(value.to_string());
            i += 1;
            continue;
        }
        if arg == "--url" && i + 1 < args.len() {
            url = Some(args[i + 1].clone());
            i += 2;
            continue;
        }
        if arg == "--token" && i + 1 < args.len() {
            token = Some(args[i + 1].clone());
            i += 2;
            continue;
        }
        i += 1;
    }

    GlobalOpts {
        json: false,
        url,
        token,
    }
}

fn parse_subcommand_project_from_args(args: &[String]) -> Option<String> {
    let mut i = 1usize;
    while i < args.len() {
        let arg = &args[i];
        if let Some(value) = arg.strip_prefix("--project=") {
            return Some(value.to_string());
        }
        if arg == "--project" && i + 1 < args.len() {
            return Some(args[i + 1].clone());
        }
        i += 1;
    }
    None
}

async fn build_dynamic_after_help(
    client: &YouTrackClient,
    subcommand: &str,
    project: Option<&str>,
) -> String {
    if subcommand == "list" {
        return build_list_after_help(client, project).await;
    }
    if subcommand == "update" {
        return build_update_after_help(client, project).await;
    }
    if subcommand == "create" {
        return build_create_after_help(client, project).await;
    }
    String::new()
}

async fn build_list_after_help(client: &YouTrackClient, project: Option<&str>) -> String {
    build_field_help(client, project, "--filter").await
}

async fn build_update_after_help(client: &YouTrackClient, project: Option<&str>) -> String {
    build_field_help(client, project, "--set").await
}

async fn build_create_after_help(client: &YouTrackClient, project: Option<&str>) -> String {
    build_field_help(client, project, "--field").await
}

async fn build_field_help(client: &YouTrackClient, project: Option<&str>, flag: &str) -> String {
    let projects = client.list_project_values().await;
    let (notice, fields) = if let Some(project_key) = project {
        (
            String::new(),
            client
                .list_project_custom_field_suggestions(project_key)
                .await,
        )
    } else {
        (
            "Project-scoped values require --project <PROJECT>.\n".to_string(),
            Ok(Vec::new()),
        )
    };

    format!(
        "{notice} Field values project specifical {}:\n {field_help}",
        summarize_values(projects),
        field_help = summarize_field_suggestions(fields),
    )
}

fn summarize_values(result: Result<Vec<String>>) -> String {
    match result {
        Ok(values) if values.is_empty() => "(none)".to_string(),
        Ok(values) => {
            let limit = 12usize;
            if values.len() > limit {
                format!(
                    "{} ... (+{} more)",
                    values[..limit].join(", "),
                    values.len() - limit
                )
            } else {
                values.join(", ")
            }
        }
        Err(err) => format!("error: {err}"),
    }
}

fn summarize_field_suggestions(result: Result<Vec<ProjectFieldSuggestion>>) -> String {
    match result {
        Ok(fields) if fields.is_empty() => "  custom fields: (none)".to_string(),
        Ok(fields) => {
            let mut lines = Vec::with_capacity(fields.len() + 1);
            lines.push("  custom fields:".to_string());
            for field in fields {
                lines.push(format!(
                    "    {} => {}",
                    field.name,
                    summarize_plain_values(&field.values)
                ));
            }
            lines.join("\n")
        }
        Err(err) => format!("  custom fields: error: {err}"),
    }
}

fn summarize_plain_values(values: &[String]) -> String {
    if values.is_empty() {
        return "(none)".to_string();
    }
    let limit = 12usize;
    if values.len() > limit {
        format!(
            "{} ... (+{} more)",
            values[..limit].join(", "),
            values.len() - limit
        )
    } else {
        values.join(", ")
    }
}

fn build_client(global: &GlobalOpts) -> Result<YouTrackClient> {
    let config = Config::load()?;
    let url = config.resolve_url(global.url.as_deref())?;
    let token = config.resolve_token(global.token.as_deref())?;
    YouTrackClient::new(&url, &token)
}

#[derive(Serialize, Tabled)]
struct MeRow {
    id: String,
    login: String,
    full_name: String,
    email: String,
}

fn render_me(me: &api::models::Me, as_json: bool) -> Result<()> {
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

fn render_projects(projects: &[api::models::Project], as_json: bool) -> Result<()> {
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
struct IssueRow {
    id_readable: String,
    summary: String,
    project: String,
    assignee: String,
    state: String,
    priority: String,
    issue_type: String,
    updated: String,
}

fn render_issues(issues: &[api::models::Issue], as_json: bool) -> Result<()> {
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
            assignee: issue_custom_field(issue, "Assignee"),
            state: issue_custom_field(issue, "State"),
            priority: issue_custom_field(issue, "Priority"),
            issue_type: issue_custom_field(issue, "Type"),
            updated: issue.updated.map(|t| t.to_string()).unwrap_or_default(),
        })
        .collect();

    println!("{}", format_table(&rows));
    Ok(())
}

fn render_issue_detail(issue: &api::models::Issue, as_json: bool) -> Result<()> {
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
    println!("assignee: {}", issue_custom_field(issue, "Assignee"));
    println!("state: {}", issue_custom_field(issue, "State"));
    println!("priority: {}", issue_custom_field(issue, "Priority"));
    println!("type: {}", issue_custom_field(issue, "Type"));
    if let Some(tags) = &issue.tags {
        let names: Vec<String> = tags.iter().map(|tag| opt_str(&tag.name)).collect();
        println!("tags: {}", names.join(", "));
    }
    render_issue_links(issue);

    Ok(())
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

#[derive(Serialize, Tabled)]
struct CommentRow {
    id: String,
    text: String,
    created: String,
}

fn render_comment(comment: &api::models::IssueComment, as_json: bool) -> Result<()> {
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

fn opt_str(value: &Option<String>) -> String {
    value.clone().unwrap_or_default()
}

fn opt_nested_str(value: &Option<Option<String>>) -> String {
    value.clone().flatten().unwrap_or_default()
}

fn build_issue_query(project: &str, filters: &[(String, String)]) -> String {
    let mut parts = vec![format!("project:{}", quote_query_value(project))];

    for (key, value) in filters {
        parts.push(format!(
            "{}:{}",
            quote_query_field_name(key),
            quote_query_value(value)
        ));
    }

    parts.join(" ")
}

fn quote_query_field_name(value: &str) -> String {
    if value.chars().any(|c| c.is_whitespace() || c == '"') {
        format!("\"{}\"", value.replace('"', "\\\""))
    } else {
        value.to_string()
    }
}

fn quote_query_value(value: &str) -> String {
    if value.chars().any(char::is_whitespace) {
        format!("\"{}\"", value.replace('"', "\\\""))
    } else {
        value.to_string()
    }
}

fn parse_key_value_specs(values: &[String], flag_name: &str) -> Result<Vec<(String, String)>> {
    values
        .iter()
        .map(|value| parse_key_value_spec(value, flag_name))
        .collect()
}

fn parse_key_value_spec(value: &str, flag_name: &str) -> Result<(String, String)> {
    let Some((key, raw_value)) = value.split_once('=') else {
        return Err(TrackItError::Config(format!(
            "Invalid {flag_name} format '{value}'. Expected KEY=VALUE"
        )));
    };

    let key = key.trim();
    let parsed_value = raw_value.trim();
    if key.is_empty() || parsed_value.is_empty() {
        return Err(TrackItError::Config(format!(
            "Invalid {flag_name} format '{value}'. Both KEY and VALUE must be non-empty"
        )));
    }

    Ok((key.to_string(), parsed_value.to_string()))
}

fn validate_field_keys(
    assignments: &[(String, String)],
    fields: &[ProjectFieldSuggestion],
    flag_name: &str,
) -> Result<()> {
    let mut known_fields: Vec<String> = fields.iter().map(|f| f.name.clone()).collect();
    known_fields.sort_by_key(|v| v.to_ascii_lowercase());

    for (key, _) in assignments {
        if fields.iter().any(|f| f.name.eq_ignore_ascii_case(key)) {
            continue;
        }
        let available = if known_fields.is_empty() {
            "(none)".to_string()
        } else {
            known_fields.join(", ")
        };
        return Err(TrackItError::Config(format!(
            "Unknown field '{key}' for {flag_name}. Available fields in this project: {available}"
        )));
    }

    Ok(())
}

fn parse_link_spec(value: &str) -> Result<(String, String)> {
    let Some((relation, issue)) = value.split_once(':') else {
        return Err(TrackItError::Config(format!(
            "Invalid link format '{value}'. Expected RELATION:ISSUE, e.g. 'relates to:PRJ-123'"
        )));
    };

    let relation = relation.trim();
    let issue = issue.trim();
    if relation.is_empty() || issue.is_empty() {
        return Err(TrackItError::Config(format!(
            "Invalid link format '{value}'. Both relation and issue must be non-empty"
        )));
    }

    Ok((relation.to_string(), issue.to_string()))
}

fn issue_custom_field(issue: &api::models::Issue, field_name: &str) -> String {
    let Some(custom_fields) = &issue.custom_fields else {
        return String::new();
    };

    for field in custom_fields {
        let (name, value) = issue_custom_field_parts(field);
        if name
            .map(|n| n.eq_ignore_ascii_case(field_name))
            .unwrap_or(false)
        {
            if let Some(value) = value {
                return custom_field_value_to_string(value);
            }
            return String::new();
        }
    }

    String::new()
}

fn issue_custom_field_parts(
    field: &api::models::IssueCustomField,
) -> (Option<&String>, Option<&serde_json::Value>) {
    use api::models::IssueCustomField::*;

    match field {
        DateIssueCustomField { name, value, .. }
        | IssueCustomField { name, value, .. }
        | MultiBuildIssueCustomField { name, value, .. }
        | MultiEnumIssueCustomField { name, value, .. }
        | MultiGroupIssueCustomField { name, value, .. }
        | MultiOwnedIssueCustomField { name, value, .. }
        | MultiUserIssueCustomField { name, value, .. }
        | DatabaseMultiValueIssueCustomField { name, value, .. }
        | MultiVersionIssueCustomField { name, value, .. }
        | PeriodIssueCustomField { name, value, .. }
        | SimpleIssueCustomField { name, value, .. }
        | SingleBuildIssueCustomField { name, value, .. }
        | SingleEnumIssueCustomField { name, value, .. }
        | SingleGroupIssueCustomField { name, value, .. }
        | SingleOwnedIssueCustomField { name, value, .. }
        | SingleUserIssueCustomField { name, value, .. }
        | DatabaseSingleValueIssueCustomField { name, value, .. }
        | SingleVersionIssueCustomField { name, value, .. }
        | StateIssueCustomField { name, value, .. }
        | StateMachineIssueCustomField { name, value, .. }
        | TextIssueCustomField { name, value, .. } => (name.as_ref(), value.as_ref()),
    }
}

fn custom_field_value_to_string(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Null => String::new(),
        serde_json::Value::Bool(v) => v.to_string(),
        serde_json::Value::Number(v) => v.to_string(),
        serde_json::Value::String(v) => v.clone(),
        serde_json::Value::Array(values) => values
            .iter()
            .map(custom_field_value_to_string)
            .filter(|v| !v.is_empty())
            .collect::<Vec<_>>()
            .join(", "),
        serde_json::Value::Object(map) => {
            for key in ["name", "fullName", "login", "idReadable", "id"] {
                if let Some(v) = map.get(key) {
                    let text = custom_field_value_to_string(v);
                    if !text.is_empty() {
                        return text;
                    }
                }
            }
            serde_json::to_string(value).unwrap_or_default()
        }
    }
}
