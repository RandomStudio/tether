use env_logger::{Builder, Env};
use log::{debug, info};
use tether_agent::{PlugOptionsBuilder, TetherAgentOptionsBuilder};

use clap::{Args, Parser, Subcommand};

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

#[derive(Args)]
struct ReceiveOptions {
    #[arg(long = "topic", default_value_t=String::from("#"))]
    subscribe_topic: String,
}

#[derive(Args)]
struct SendOptions {
    /// Specify an Agent Role; this will be used for the auto-generated publish topic
    /// (ignored if you provide your own plug.topic)
    #[arg(long = "agent.role", default_value_t=String::from(defaults::AGENT_ROLE))]
    agent_role: String,

    /// Specify an Agent ID or Group; this will be used for the auto-generated publish topic
    /// (ignored if you provide your own plug.topic)
    #[arg(long = "agent.id", default_value_t=String::from(defaults::AGENT_ID))]
    agent_id: String,

    /// Overide the auto-generated topic with your own, to use with every published message
    #[arg(long = "plug.name", default_value_t=String::from("testMessages"))]
    plug_name: String,

    /// Overide the auto-generated topic with your own, to use with every published message
    #[arg(long = "plug.topic")]
    plug_topic: Option<String>,

    /// Optionally provide a custom message. Provide this as a valid JSON string.
    #[arg(long = "message")]
    custom_message: Option<String>,
}

#[derive(Args)]
struct TopicOptions {
    #[arg(long = "topic", default_value_t=String::from("#"))]
    subscribe_topic: String,
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

fn receive(cli: &Cli, options: &ReceiveOptions) {
    info!("Tether Receive Utility");
    let tether_agent = TetherAgentOptionsBuilder::new(defaults::AGENT_ROLE)
        .host(&cli.tether_host)
        .port(cli.tether_port)
        .username(&cli.tether_username)
        .password(&cli.tether_password)
        .build()
        .expect("failed to connect Tether");

    let _input = PlugOptionsBuilder::create_input("all")
        .topic(&options.subscribe_topic)
        .build(&tether_agent);

    loop {
        while let Some((plug_name, message)) = tether_agent.check_messages() {
            debug!("Received message on plug {}: {:?}", plug_name, message);
            info!("Received message on topic \"{}\"", message.topic());
            let bytes = message.payload();
            if bytes.is_empty() {
                info!("Empty message payload");
            } else {
                let value: rmpv::Value =
                    rmp_serde::from_slice(bytes).expect("failed to decode msgpack");
                let json = serde_json::to_string(&value).expect("failed to stringify JSON");
                info!("Decoded MessagePack payload: {}", json);
            }
        }
    }
}

fn send(cli: &Cli, options: &SendOptions) {
    info!("Tether Send Utility");
}

fn topics(cli: &Cli, options: &TopicOptions) {
    info!("Tether Topics Parsing Utility");
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
