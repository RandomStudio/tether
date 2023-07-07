use env_logger::{Builder, Env};
use log::{debug, error, warn};

use clap::{Parser, Subcommand};

mod tether_playback;
mod tether_receive;
mod tether_record;
mod tether_send;
mod tether_topics;

use tether_agent::TetherAgentOptionsBuilder;
use tether_playback::PlaybackOptions;
use tether_receive::{receive, ReceiveOptions};
use tether_record::RecordOptions;
use tether_send::{send, SendOptions};
use tether_topics::{topics, TopicOptions};

use crate::{tether_playback::playback, tether_record::record};

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

    /// Role to use for any auto-generated topics on publish
    #[arg(long = "tether.role", default_value_t=String::from("utils"))]
    pub tether_role: String,

    /// ID/Group to use for any auto-generated topics on publish
    #[arg(long = "tether.id", default_value_t=String::from("any"))]
    pub tether_id: String,

    #[arg(long = "loglevel",default_value_t=String::from("info"))]
    pub log_level: String,
}

#[derive(Subcommand)]
enum Commands {
    Receive(ReceiveOptions),
    Send(SendOptions),
    Topics(TopicOptions),
    Playback(PlaybackOptions),
    Record(RecordOptions),
}

fn main() {
    let cli = Cli::parse();

    let mut env_builder = Builder::from_env(Env::default().default_filter_or(&cli.log_level));
    env_builder.init();

    debug!("Debugging is enabled; could be verbose");

    let tether_agent = TetherAgentOptionsBuilder::new(&cli.tether_role)
        .id(&cli.tether_id)
        .host(&cli.tether_host)
        .port(cli.tether_port)
        .username(&cli.tether_username)
        .password(&cli.tether_password)
        .build()
        .unwrap_or_else(|_| {
            error!("Failed to initialise and/or connect the Tether Agent");
            warn!(
                "Check your Tether settings and ensure that you have a correctly-configured MQTT broker running at {}:{}",
                cli.tether_host, cli.tether_port
            );
            panic!("Failed to init/connect Tether Agent")
        });

    match &cli.command {
        Commands::Receive(options) => receive(options, &tether_agent),
        Commands::Send(options) => send(options, &tether_agent),
        Commands::Topics(options) => topics(options, &tether_agent),
        Commands::Playback(options) => playback(options, &tether_agent),
        Commands::Record(options) => record(options, &tether_agent),
    }
}
