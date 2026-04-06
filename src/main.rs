mod app;
mod cli;
mod config;
mod error;
mod output;
mod youtrack;

use clap::Parser;

use crate::app::args::{Commands, IssueCommands, ProjectCommands, ProjectGetCommands};
use crate::app::context::build_client;
use crate::app::parsing::{
    build_issue_query, parse_key_value_specs, parse_link_spec, summarize_plain_values,
};
use crate::app::render_basic::{
    render_comment, render_me, render_project_custom_fields, render_projects,
};
use crate::app::render_issue::{render_issue_detail, render_issues};
use crate::cli::run_setup_wizard;
use crate::error::TrackItError;
use app::args::Cli;
use app::context::GlobalOpts;
use error::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let global = GlobalOpts {
        json: cli.json,
        url: cli.url.clone(),
        token: cli.token.clone(),
    };

    let command = cli.command;
    match command {
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
                ProjectCommands::Get { command } => match command {
                    ProjectGetCommands::CustomField { project } => {
                        let fields = client
                            .list_project_custom_field_suggestions(&project)
                            .await?;
                        render_project_custom_fields(&fields, global.json, summarize_plain_values)?;
                    }
                },
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
                    let query = build_issue_query(project.as_deref(), &parsed_filters);
                    let issues = client.list_issues(query.as_deref(), skip, top).await?;
                    render_issues(&issues, global.json)?;
                }
                IssueCommands::Get { id } => {
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
                    let assignments = parse_key_value_specs(&args.field, "--field")?;
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
