use crossterm::{
    cursor::{RestorePosition, SavePosition},
    execute,
    style::Print,
};
use env_logger::{Builder, Env};
use log::*;

use clap::{Parser, Subcommand};

use tether_agent::TetherAgentOptionsBuilder;
use tether_utils::*;

use std::{
    io::{stdout, Write},
    time::{Duration, SystemTime},
};

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(long = "protocol", default_value_t=String::from("tcp"))]
    pub tether_protocol: String,

    #[arg(long = "host", default_value_t=String::from("localhost"))]
    pub tether_host: String,

    #[arg(long = "port", default_value_t = 1883)]
    pub tether_port: u16,

    #[arg(long = "path", default_value_t=String::from("/"))]
    pub tether_base_path: String,

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
    std::io::stdout().flush().unwrap();

    let mut env_builder = Builder::from_env(Env::default().default_filter_or(&cli.log_level));
    env_builder.filter_module("paho_mqtt", LevelFilter::Warn);
    env_builder.init();

    debug!("Debugging is enabled; could be verbose");

    let mut tether_agent = TetherAgentOptionsBuilder::new(&cli.tether_role)
        .id(Some(cli.tether_id).as_deref())
        .protocol(Some(cli.tether_protocol).as_deref())
        .host(Some(cli.tether_host.clone()).as_deref())
        .port(Some(cli.tether_port))
        .base_path(Some(cli.tether_base_path).as_deref())
        .username(Some(cli.tether_username).as_deref())
        .password(Some(cli.tether_password).as_deref())
        .build()
        .unwrap_or_else(|_| {
            error!("Failed to initialise and/or connect the Tether Agent");
            warn!(
                "Check your Tether settings and ensure that you have a correctly-configured MQTT broker running at {}:{}",
                &cli.tether_host, cli.tether_port
            );
            panic!("Failed to init/connect Tether Agent")
        });

    match &cli.command {
        Commands::Receive(options) => {
            tether_receive::receive(options, &mut tether_agent, |_plug_name, topic, decoded| {
                let contents = decoded.unwrap_or("(empty/invalid message)".into());
                info!("Received on topic \"{}\" :: \n{}\n", topic, contents);
            })
        }
        Commands::Send(options) => {
            tether_send::send(options, &mut tether_agent)
                .unwrap_or_else(|e| error!("Failed to send: {}", e));
            // This silliness (below) is only needed because the command might
            // finish before the message is even sent!
            // Less of an issue with long-running applications, and should
            // be properly solved with an async version.
            std::thread::sleep(std::time::Duration::from_millis(1000));
        }
        Commands::Topics(options) => {
            let mut insights = tether_topics::insights::Insights::new(options, &mut tether_agent);
            let mut last_update = SystemTime::now();

            loop {
                if !insights.sample() {
                    std::thread::sleep(Duration::from_millis(1));
                }
                while let Some((topic, payload)) = tether_agent.check_messages() {
                    let topics_did_udate = insights.update(&topic, payload);
                    print_insights_summary(&insights, topics_did_udate, options.graph_enable);
                }
                if let Ok(elapsed) = last_update.elapsed() {
                    if elapsed > Duration::from_secs(1) {
                        print_insights_summary(&insights, false, options.graph_enable);
                        last_update = SystemTime::now();
                    }
                }
            }
        }
        Commands::Playback(options) => {
            let player = tether_playback::TetherPlaybackUtil::new(options.clone());
            player.start(&tether_agent);
        }
        Commands::Record(options) => {
            let recorder = tether_record::TetherRecordUtil::new(options.clone());
            recorder.start_recording(&mut tether_agent);
        }
    }
}

fn print_insights_summary(
    insights: &tether_topics::insights::Insights,
    topics_did_update: bool,
    enable_graph: bool,
) {
    if topics_did_update {
        info!(
            "Topics update-------------------------------\n{}\n",
            &insights
        );
    }
    let mut stdout = stdout();
    let rate_string = match insights.get_rate() {
        Some(r) => format!("{:.1} msg/s", r),
        None => String::from("unknown"),
    };
    let sampler_graph: String = {
        if !enable_graph {
            String::from("")
        } else {
            insights
                .sampler()
                .delta_entries()
                .iter()
                .fold(String::from(""), |acc, x| {
                    let symbol = match *x {
                        0 => ".",
                        1 => "_",
                        _ => "+",
                    };
                    // if *x > 0 { "+" } else { "." };
                    format!("{} {}", acc, symbol)
                })
        }
    };
    if enable_graph {
        execute!(
            stdout,
            SavePosition,
            Print(format!(
                "Live message count: {}             \n",
                insights.message_count()
            )),
            Print(format!("Message rate: {}         \n", rate_string)),
            // Print(format!("Sampler: {:?}", insights.sampler().delta_entries())),
            Print(sampler_graph),
            RestorePosition,
        )
        .unwrap();
    }
    // disable_raw_mode();
}
