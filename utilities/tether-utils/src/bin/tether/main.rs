use env_logger::{Builder, Env};
use log::*;

use clap::{Parser, Subcommand};

use tether_agent::TetherAgentOptionsBuilder;
use tether_utils::*;

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
    Receive(tether_receive::ReceiveOptions),
    Send(tether_send::SendOptions),
    Topics(tether_topics::TopicOptions),
    Playback(tether_playback::PlaybackOptions),
    Record(tether_record::RecordOptions),
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
        Commands::Receive(options) => {
            tether_receive::receive(options, &tether_agent, |_plug_name, message, decoded| {
                let contents = decoded.unwrap_or("(empty/invalid message)".into());
                info!(
                    "Received on topic \"{}\" :: \n{}\n",
                    message.topic(),
                    contents
                );
            })
        }
        Commands::Send(options) => tether_send::send(options, &tether_agent)
            .unwrap_or_else(|e| error!("Failed to send: {}", e)),
        Commands::Topics(options) => {
            let mut insights = tether_topics::Insights::new(&options, &tether_agent);

            loop {
                while let Some((_plug_name, message)) = tether_agent.check_messages() {
                    if insights.update(&message) {
                        info!("Insights update: \n{}", insights);
                    }
                }
            }
        }
        Commands::Playback(options) => tether_playback::playback(options, &tether_agent),
        Commands::Record(options) => {
            let recorder = tether_record::TetherRecordUtil::new(options, tether_agent);
            recorder.start_recording();
        }
    }
}
