use crossterm::{
    cursor::{Hide, RestorePosition, SavePosition},
    execute,
    style::Print,
};
use env_logger::{Builder, Env};
use log::*;

use clap::{Parser, Subcommand};

use tether_agent::{mqtt::topic, TetherAgentOptionsBuilder};
use tether_utils::{tether_playback::TetherPlaybackUtil, tether_topics::Insights, *};

use std::{
    io::stdout,
    time::{Duration, SystemTime},
};

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(long = "host", default_value_t=String::from("localhost"))]
    pub tether_host: String,

    #[arg(long = "port", default_value_t = 1883)]
    pub tether_port: u16,

    #[arg(long = "username", default_value_t=String::from("tether"))]
    pub tether_username: String,

    #[arg(long = "password", default_value_t=String::from("sp_ceB0ss!"))]
    pub tether_password: String,

    /// Role to use for any auto-generated topics on publish
    #[arg(long = "role", default_value_t=String::from("utils"))]
    pub tether_role: String,

    /// ID/Group to use for any auto-generated topics on publish
    #[arg(long = "id", default_value_t=String::from("any"))]
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
            let mut insights = tether_topics::Insights::new(options, &tether_agent);
            let mut last_update = SystemTime::now();

            loop {
                while let Some((_plug_name, message)) = tether_agent.check_messages() {
                    let topics_did_udate = insights.update(&message);
                    print_insights_summary(&insights, topics_did_udate);
                }
                if let Ok(elapsed) = last_update.elapsed() {
                    if elapsed > Duration::from_secs(1) {
                        print_insights_summary(&insights, false);
                        last_update = SystemTime::now();
                    }
                }
            }
        }
        Commands::Playback(options) => {
            let player = TetherPlaybackUtil::new(options.clone());
            player.start(&tether_agent);
        }
        Commands::Record(options) => {
            let recorder = tether_record::TetherRecordUtil::new(options.clone());
            recorder.start_recording(&tether_agent);
        }
    }
}

fn print_insights_summary(insights: &Insights, topics_did_update: bool) {
    if topics_did_update {
        info!("\nTopics update\n------------\n{}", &insights);
    }
    let mut stdout = stdout();
    let rate_string = match insights.get_rate() {
        Some(r) => format!("{:.1} msg/s", r),
        None => String::from("unknown"),
    };
    execute!(
        stdout,
        SavePosition,
        Print(format!(
            "Live message count: {}\n",
            insights.message_count()
        )),
        Print(format!("Message rate: {}         \n", rate_string)),
        RestorePosition,
    )
    .unwrap();
}
