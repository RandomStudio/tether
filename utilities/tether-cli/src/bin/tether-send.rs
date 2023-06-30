use cli_shared::defaults::{AGENT_ID, AGENT_ROLE};
use env_logger::{Builder, Env};
use log::{debug, error, info, warn};
use serde::Serialize;
use tether_agent::{PlugOptionsBuilder, TetherAgentOptionsBuilder};

#[derive(Serialize, Debug)]
struct DummyData {
    id: usize,
    a_float: f32,
    an_int_array: Vec<usize>,
    a_string: String,
}

use clap::{command, Parser};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// Specify an Agent Role; this will be used for the auto-generated publish topic
    /// (ignored if you provide your own plug.topic)
    #[arg(long = "agent.role", default_value_t=String::from(AGENT_ROLE))]
    pub agent_role: String,

    /// Specify an Agent ID or Group; this will be used for the auto-generated publish topic
    /// (ignored if you provide your own plug.topic)
    #[arg(long = "agent.id", default_value_t=String::from(AGENT_ID))]
    pub agent_id: String,

    /// Overide the auto-generated topic with your own, to use with every published message
    #[arg(long = "plug.name", default_value_t=String::from("testMessages"))]
    pub plug_name: String,

    /// Overide the auto-generated topic with your own, to use with every published message
    #[arg(long = "plug.topic")]
    pub plug_topic: Option<String>,

    #[arg(long = "loglevel",default_value_t=String::from("info"))]
    pub log_level: String,

    /// Optionally provide a custom message. Provide this as a valid JSON string.
    #[arg(long = "message")]
    pub custom_message: Option<String>,
}

fn main() {
    let cli = Cli::parse();

    let mut builder = Builder::from_env(Env::default().default_filter_or(cli.log_level));
    builder.init();

    debug!("Debugging is enabled; could be verbose");

    let publish_topic = match cli.plug_topic {
        Some(override_topic) => {
            warn!(
                "Using override topic \"{}\"; agent role, agent ID and plug name will be ignored",
                override_topic
            );
            override_topic
        }
        None => {
            let auto_generated_topic =
                format!("{}/{}/{}", &cli.agent_role, &cli.agent_id, &cli.plug_name);
            info!("Using auto-generated topic \"{}\"", &auto_generated_topic);
            auto_generated_topic
        }
    };

    let tether_agent = TetherAgentOptionsBuilder::new(&cli.agent_role)
        .id(&cli.agent_id)
        .build()
        .expect("failed to connect Tether");

    let output = PlugOptionsBuilder::create_output(&cli.plug_name)
        .topic(&publish_topic)
        .build(&tether_agent);

    if let Some(custom_message) = cli.custom_message {
        debug!(
            "Attempting to decode provided custom message \"{}\"",
            &custom_message
        );
        match serde_json::from_str::<serde_json::Value>(&custom_message) {
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
