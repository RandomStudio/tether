use clap::Args;
use log::{debug, error, info, warn};
use serde::Serialize;
use tether_agent::{PlugOptionsBuilder, TetherAgentOptionsBuilder};

use crate::{defaults, Cli};

#[derive(Args)]
pub struct SendOptions {
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

pub fn send(cli: &Cli, options: &SendOptions) {
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
