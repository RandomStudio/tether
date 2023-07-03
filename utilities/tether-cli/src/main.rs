use env_logger::{Builder, Env};
use log::debug;

use clap::{Parser, Subcommand};

mod tether_receive;
mod tether_send;
mod tether_topics;

use tether_receive::{receive, ReceiveOptions};
use tether_send::{send, SendOptions};
use tether_topics::{topics, TopicOptions};

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(long = "tether.host", default_value_t=String::from("localhost"))]
    pub tether_host: String,

    #[arg(long = "tether.port", default_value_t = 1883)]
    pub tether_port: u16,

    #[arg(long = "tether.username", default_value_t=String::from("tether"))]
    pub tether_username: String,

    #[arg(long = "tether.password", default_value_t=String::from("sp_ceB0ss!"))]
    pub tether_password: String,

    #[arg(long = "loglevel",default_value_t=String::from("info"))]
    pub log_level: String,
}

#[derive(Subcommand)]
enum Commands {
    Receive(ReceiveOptions),
    Send(SendOptions),
    Topics(TopicOptions),
}

mod defaults {
    pub const AGENT_ROLE: &str = "testAgent";
    pub const AGENT_ID: &str = "any";
}

fn main() {
    let cli = Cli::parse();

    let mut builder = Builder::from_env(Env::default().default_filter_or(&cli.log_level));
    builder.init();

    debug!("Debugging is enabled; could be verbose");

    match &cli.command {
        Commands::Receive(options) => receive(&cli, options),
        Commands::Send(options) => send(&cli, options),
        Commands::Topics(options) => topics(&cli, options),
    }
}
