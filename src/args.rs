use std::time::Duration;
use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Run the application with Channels configured by the user
    Twitch,
    /// Run the stressing test for the application
    Stress {
        #[command(flatten)]
        stress: StressConfig,
    }
}

#[derive(Args)]
pub struct StressConfig {
    #[arg(short, long, default_value = "30")]
    pub channels: usize,

    #[arg(short, long, default_value = "1", value_parser = parse_duration)]
    pub interval: Duration,

    #[arg(short, long, default_value = "4")]
    pub workers: usize,
}

#[derive(Args)]
pub struct ServerConfig {
    #[arg(short, long, default_value = "twitch")]
    pub keyspace: String,

    #[arg(long, default_value = "localhost:9042")]
    pub hostnames: Vec<String>,

    #[arg(short, long, default_value = "")]
    pub username: String,

    #[arg(short, long, default_value = "")]
    pub password: String,

    #[arg(short, long, default_value = "2", value_parser = parse_duration)]
    pub timeout: Duration,
}

fn parse_duration(arg: &str) -> Result<Duration, std::num::ParseIntError> {
    let secs = arg.parse()?;
    Ok(Duration::from_secs(secs))
}