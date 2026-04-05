mod cli;
mod config;
mod error;
mod output;
mod youtrack;

use clap::{ArgAction, Args, Parser, Subcommand};
use cli::run_setup_wizard;
use serde::Serialize;
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

    #[arg(long, global = true, help = "Use specific config file")]
    config: Option<String>,

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
    config: Option<String>,
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
        query: Option<String>,
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

    #[command(about = "Assign issue")]
    Assign {
        id: String,
        #[arg(long)]
        user: String,
    },

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
    state: Option<String>,
    #[arg(long)]
    summary: Option<String>,
    #[arg(long)]
    description: Option<String>,
    #[arg(long = "tag", action = ArgAction::Append)]
    tags: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let global = GlobalOpts {
        json: cli.json,
        config: cli.config.clone(),
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
                IssueCommands::List { query, skip, top } => {
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
                IssueCommands::Assign { id, user } => {
                    client.assign_issue(&id, &user).await?;
                    let issue = client.get_issue(&id).await?;
                    render_issue_detail(&issue, global.json)?;
                }
                IssueCommands::Comment { id, text } => {
                    let comment = client.comment_issue(&id, &text).await?;
                    render_comment(&comment, global.json)?;
                }
                IssueCommands::Update(args) => {
                    if args.state.is_none()
                        && args.summary.is_none()
                        && args.description.is_none()
                        && args.tags.is_empty()
                    {
                        return Err(TrackItError::Config(
                            "issues update needs at least one of: --state, --summary, --description, --tag"
                                .to_string(),
                        ));
                    }

                    if let Some(state) = args.state.as_deref() {
                        client.update_issue_state(&args.id, state).await?;
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

                    for tag in &args.tags {
                        client.add_tag(&args.id, tag).await?;
                    }

                    let issue = client.get_issue(&args.id).await?;
                    render_issue_detail(&issue, global.json)?;
                }
            }
        }
    }

    Ok(())
}

fn build_client(global: &GlobalOpts) -> Result<YouTrackClient> {
    let config = load_config(global)?;
    let url = config.resolve_url(global.url.as_deref())?;
    let token = config.resolve_token(global.token.as_deref())?;
    YouTrackClient::new(&url, &token)
}

fn load_config(global: &GlobalOpts) -> Result<Config> {
    if let Some(path) = &global.config {
        let content = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        return Ok(config);
    }

    Config::load()
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
    id: String,
    id_readable: String,
    summary: String,
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
            id: opt_str(&issue.id),
            id_readable: opt_str(&issue.id_readable),
            summary: opt_nested_str(&issue.summary),
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
