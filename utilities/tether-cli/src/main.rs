use env_logger::{Builder, Env};
use log::{debug, error, info, warn};
use serde::Serialize;
use tether_agent::{
    mqtt::Message, parse_agent_id, parse_agent_role, parse_plug_name, PlugOptionsBuilder,
    TetherAgentOptionsBuilder,
};

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

#[derive(Serialize, Debug)]
struct DummyData {
    id: usize,
    a_float: f32,
    an_int_array: Vec<usize>,
    a_string: String,
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

#[derive(Debug)]
pub struct Insights {
    topics: Vec<String>,
    roles: Vec<String>,
    ids: Vec<String>,
    plugs: Vec<String>,
    // message_count: u64,
}

impl Insights {
    fn new() -> Self {
        Insights {
            topics: Vec::new(),
            roles: Vec::new(),
            ids: Vec::new(),
            plugs: Vec::new(),
            // message_count: 0,
        }
    }

    pub fn update(&mut self, _plug_name: &str, message: &Message) {
        // self.message_count += 1;

        // Collect some stats...
        add_if_unique(message.topic(), &mut self.topics);
        add_if_unique(
            parse_agent_role(message.topic()).unwrap_or("unknown"),
            &mut self.roles,
        );
        add_if_unique(
            parse_agent_id(message.topic()).unwrap_or("unknown"),
            &mut self.ids,
        );
        add_if_unique(
            parse_plug_name(message.topic()).unwrap_or("unknown"),
            &mut self.plugs,
        );
    }
}

fn add_if_unique(item: &str, list: &mut Vec<String>) {
    if !list.iter().any(|i| i.eq(item)) {
        list.push(String::from(item));
    }
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

    let publish_topic = match &options.plug_topic {
        Some(override_topic) => {
            warn!(
                "Using override topic \"{}\"; agent role, agent ID and plug name will be ignored",
                override_topic
            );
            String::from(override_topic)
        }
        None => {
            let auto_generated_topic: String = format!(
                "{}/{}/{}",
                &options.agent_role, &options.agent_id, &options.plug_name
            );
            info!("Using auto-generated topic \"{}\"", &auto_generated_topic);
            auto_generated_topic
        }
    };

    let tether_agent = TetherAgentOptionsBuilder::new(defaults::AGENT_ROLE)
        .host(&cli.tether_host)
        .port(cli.tether_port)
        .username(&cli.tether_username)
        .password(&cli.tether_password)
        .build()
        .expect("failed to connect Tether");

    let output = PlugOptionsBuilder::create_output(&options.plug_name)
        .topic(&publish_topic)
        .build(&tether_agent);

    if let Some(custom_message) = &options.custom_message {
        debug!(
            "Attempting to decode provided custom message \"{}\"",
            &custom_message
        );
        match serde_json::from_str::<serde_json::Value>(custom_message) {
            Ok(encoded) => {
                let payload = rmp_serde::to_vec_named(&encoded).expect("failed to encode msgpack");
                tether_agent
                    .publish(&output, Some(&payload))
                    .expect("failed to publish");
            }
            Err(e) => {
                error!("Could not serialise String -> JSON; error: {}", e);
            }
        }
    } else {
        let payload = DummyData {
            id: 0,
            a_float: 42.0,
            an_int_array: vec![1, 2, 3, 4],
            a_string: "hello world".into(),
        };
        info!("Sending dummy data {:?}", payload);
        tether_agent
            .encode_and_publish(&output, &payload)
            .expect("failed to publish");
    }
}

fn topics(cli: &Cli, options: &TopicOptions) {
    info!("Tether Topics Parsing Utility");

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

    let mut insights = Insights::new();

    loop {
        while let Some((plug_name, message)) = tether_agent.check_messages() {
            insights.update(&plug_name, &message);
            info!("{:#?}", insights);
        }
    }
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
