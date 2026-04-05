mod error;
mod config;
mod client;
mod output;
mod cli;

use clap::{Parser, Subcommand};
use cli::{ConfigCommand, handle_config_command};

#[derive(Parser)]
#[command(name = "trackit")]
#[command(about = "YouTrack CLI tool", long_about = None)]
#[command(version)]
struct Cli {
    #[arg(long, global = true, help = "Output in JSON format")]
    json: bool,

    #[arg(long, global = true, help = "Use specific config file")]
    config: Option<String>,

    #[arg(long, global = true, help = "Override YouTrack URL", env = "YOUTRACK_URL")]
    url: Option<String>,

    #[arg(long, global = true, help = "Override API token", env = "YOUTRACK_TOKEN")]
    token: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Configuration management")]
    Config {
        #[command(subcommand)]
        command: ConfigCommand,
    },
}

#[tokio::main]
async fn main() -> error::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Config { command } => {
            handle_config_command(command).await?;
        }
    }

    Ok(())
}
