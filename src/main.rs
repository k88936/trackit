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
use crate::youtrack::YouTrackClient;

#[derive(Parser)]
#[command(name = "trackit")]
#[command(about = "YouTrack CLI tool", long_about = None)]
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
        project: Option<String>,
        #[arg(long)]
        state: Option<String>,
        #[arg(long)]
        assignee: Option<String>,
        #[arg(long)]
        priority: Option<String>,
        #[arg(long = "type")]
        issue_type: Option<String>,
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
}

#[derive(Args)]
struct IssueUpdateArgs {
    id: String,
    #[arg(long)]
    project: Option<String>,
    #[arg(long)]
    state: Option<String>,
    #[arg(long)]
    assignee: Option<String>,
    #[arg(long)]
    priority: Option<String>,
    #[arg(long = "type")]
    issue_type: Option<String>,
    #[arg(long)]
    summary: Option<String>,
    #[arg(long)]
    description: Option<String>,
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
                    state,
                    assignee,
                    priority,
                    issue_type,
                    skip,
                    top,
                } => {
                    let query = build_issue_query(
                        project.as_deref(),
                        state.as_deref(),
                        assignee.as_deref(),
                        priority.as_deref(),
                        issue_type.as_deref(),
                    );
                    let issues = client.list_issues(query.as_deref(), skip, top).await?;
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
                    if args.project.is_none()
                        && args.state.is_none()
                        && args.assignee.is_none()
                        && args.priority.is_none()
                        && args.issue_type.is_none()
                        && args.summary.is_none()
                        && args.description.is_none()
                    {
                        return Err(TrackItError::Config(
                            "issues update needs at least one of: --project, --state, --assignee, --priority, --type, --summary, --description"
                                .to_string(),
                        ));
                    }

                    if let Some(project) = args.project.as_deref() {
                        client.update_issue_project(&args.id, project).await?;
                    }
                    if let Some(state) = args.state.as_deref() {
                        client.update_issue_state(&args.id, state).await?;
                    }
                    if let Some(assignee) = args.assignee.as_deref() {
                        client.assign_issue(&args.id, assignee).await?;
                    }
                    if let Some(priority) = args.priority.as_deref() {
                        client.update_issue_priority(&args.id, priority).await?;
                    }
                    if let Some(issue_type) = args.issue_type.as_deref() {
                        client.update_issue_type(&args.id, issue_type).await?;
                    }

                    if args.summary.is_some() || args.description.is_some() {
                        client
                            .update_issue_text(
                                &args.id,
                                args.summary.as_deref(),
                                args.description.as_deref(),
                            )
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
        let after_help = match build_client(&global) {
            Ok(client) => build_dynamic_after_help(&client, sub).await,
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

async fn build_dynamic_after_help(client: &YouTrackClient, subcommand: &str) -> String {
    if subcommand == "list" {
        return build_list_after_help(client).await;
    }
    if subcommand == "update" {
        return build_update_after_help(client).await;
    }
    String::new()
}

async fn build_list_after_help(client: &YouTrackClient) -> String {
    let project = client.list_project_values().await;
    let state = client.suggest_list_values("State").await;
    let assignee = client.suggest_list_values("Assignee").await;
    let priority = client.suggest_list_values("Priority").await;
    let issue_type = client.suggest_list_values("Type").await;

    format!(
        "Possible values (live from YouTrack):\n  --project: {}\n  --state: {}\n  --assignee: {}\n  --priority: {}\n  --type: {}",
        summarize_values(project),
        summarize_values(state),
        summarize_values(assignee),
        summarize_values(priority),
        summarize_values(issue_type),
    )
}

async fn build_update_after_help(client: &YouTrackClient) -> String {
    let project = client.list_project_values().await;
    let state = client.suggest_update_values("State").await;
    let assignee = client.suggest_update_values("Assignee").await;
    let priority = client.suggest_update_values("Priority").await;
    let issue_type = client.suggest_update_values("Type").await;

    format!(
        "Possible values (live from YouTrack):\n  --project: {}\n  --state: {}\n  --assignee: {}\n  --priority: {}\n  --type: {}",
        summarize_values(project),
        summarize_values(state),
        summarize_values(assignee),
        summarize_values(priority),
        summarize_values(issue_type),
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

    Ok(())
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

fn build_issue_query(
    project: Option<&str>,
    state: Option<&str>,
    assignee: Option<&str>,
    priority: Option<&str>,
    issue_type: Option<&str>,
) -> Option<String> {
    let mut parts = Vec::new();

    if let Some(project) = project {
        parts.push(format!("project:{}", quote_query_value(project)));
    }

    if let Some(state) = state {
        parts.push(format!("State:{}", quote_query_value(state)));
    }
    if let Some(assignee) = assignee {
        parts.push(format!("Assignee:{}", quote_query_value(assignee)));
    }
    if let Some(priority) = priority {
        parts.push(format!("Priority:{}", quote_query_value(priority)));
    }
    if let Some(issue_type) = issue_type {
        parts.push(format!("Type:{}", quote_query_value(issue_type)));
    }

    if parts.is_empty() {
        None
    } else {
        Some(parts.join(" "))
    }
}

fn quote_query_value(value: &str) -> String {
    if value.chars().any(char::is_whitespace) {
        format!("\"{}\"", value.replace('"', "\\\""))
    } else {
        value.to_string()
    }
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
