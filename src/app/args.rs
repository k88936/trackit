use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(name = "trackit")]
#[command(
    about = "YouTrack CLI tool",
    long_about = "Youtrack CLI Tool , especially for code agent to do issue management"
)]
#[command(version)]
pub struct Cli {
    #[arg(long, global = true, help = "Output in JSON format")]
    pub json: bool,

    #[arg(
        long,
        global = true,
        help = "Override YouTrack URL",
        env = "YOUTRACK_URL"
    )]
    pub url: Option<String>,

    #[arg(
        long,
        global = true,
        help = "Override API token",
        env = "YOUTRACK_TOKEN"
    )]
    pub token: Option<String>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(about = "Interactive setup wizard")]
    SetupWizard,
    #[command(about = "Show current authenticated user")]
    Me,
    #[command(
        name = "project",
        visible_alias = "projects",
        about = "Project operations"
    )]
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
pub enum ProjectCommands {
    #[command(about = "List projects")]
    List {
        #[arg(long, help = "Number of projects to skip before listing")]
        skip: Option<i32>,
        #[arg(long, help = "Maximum number of projects to return")]
        top: Option<i32>,
    },
    #[command(
        name = "get-custom-field",
        about = "Show custom fields for one project"
    )]
    GetCustomField {
        #[arg(help = "Project id or short name")]
        project: String,
    },
}

#[derive(Subcommand)]
pub enum IssueCommands {
    #[command(about = "List issues")]
    List {
        #[arg(long, help = "Limit results to a project id or short name")]
        project: Option<String>,
        #[arg(
            long = "filter",
            value_name = "KEY=VALUE",
            help = "Filter by field value, can be provided multiple times. possible KEY and VALUE see `trackit project get-custom-field PROJECT_NAME`"
        )]
        filters: Vec<String>,
        #[arg(long, help = "Number of issues to skip before listing")]
        skip: Option<i32>,
        #[arg(long, help = "Maximum number of issues to return")]
        top: Option<i32>,
    },
    #[command(about = "Get issue details")]
    Get {
        #[arg(help = "Issue id, for example PRJ-123")]
        id: String,
    },
    #[command(about = "Create issue")]
    Create(IssueCreateArgs),
    #[command(about = "Delete issue")]
    Delete {
        #[arg(help = "Issue id, for example PRJ-123")]
        id: String,
    },
    #[command(about = "Add comment to issue")]
    Comment {
        #[arg(help = "Issue id, for example PRJ-123")]
        id: String,
        #[arg(long, help = "Comment text")]
        text: String,
    },
    #[command(about = "Update issue info")]
    Update(IssueUpdateArgs),
}

#[derive(Args)]
pub struct IssueCreateArgs {
    #[arg(long, help = "Project id or short name where the issue will be created")]
    pub project: String,
    #[arg(long, help = "Issue summary/title")]
    pub summary: String,
    #[arg(long, help = "Issue description")]
    pub description: Option<String>,
    #[arg(
        long = "field",
        value_name = "KEY=VALUE",
        help = "Set project custom field value, can be provided multiple times. possible KEY and VALUE see `trackit project get-custom-field PROJECT_NAME`"
    )]
    pub fields: Vec<String>,
    #[arg(
        long = "link",
        value_name = "RELATION:ISSUE",
        help = "Add an issue link, can be provided multiple times. possible RELATION values include `relates to`, `duplicates`, `is duplicated by`, `depends on`, `is required for`, `subtask of`, `parent for`"
    )]
    pub links: Vec<String>,
}

#[derive(Args)]
pub struct IssueUpdateArgs {
    #[arg(help = "Issue id, for example PRJ-123")]
    pub id: String,
    #[arg(
        long = "field",
        value_name = "KEY=VALUE",
        help = "Set a custom field value, can be provided multiple times. possible KEY and VALUE see `trackit project get-custom-field PROJECT_NAME`"
    )]
    pub field: Vec<String>,
    #[arg(
        long = "link",
        value_name = "RELATION:ISSUE",
        help = "Add an issue link, can be provided multiple times. possible RELATION values include `relates to`, `duplicates`, `is duplicated by`, `depends on`, `is required for`, `subtask of`, `parent for`"
    )]
    pub link: Vec<String>,
    #[arg(
        long = "unlink",
        value_name = "RELATION:ISSUE",
        help = "Remove an issue link, can be provided multiple times. possible RELATION values include `relates to`, `duplicates`, `is duplicated by`, `depends on`, `is required for`, `subtask of`, `parent for`"
    )]
    pub unlink: Vec<String>,
}
