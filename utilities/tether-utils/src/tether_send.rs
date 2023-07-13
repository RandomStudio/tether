use clap::Args;
use log::{debug, error, info, warn};
use serde::Serialize;
use tether_agent::{build_topic, PlugOptionsBuilder, TetherAgent};

#[derive(Args)]
pub struct SendOptions {
    /// Overide the auto-generated topic with your own, to use with every published message
    #[arg(long = "plug.name", default_value_t=String::from("testMessages"))]
    pub plug_name: String,

    /// Overide the auto-generated topic with your own, to use with every published message
    #[arg(long = "plug.topic")]
    pub plug_topic: Option<String>,

    /// Provide a custom message as an escaped JSON string which will be converted
    /// into MessagePack; by default the payload will be empty.
    #[arg(long = "message")]
    pub message_payload_json: Option<String>,

    /// Flag to generate dummy data for the MessagePack payload; useful for testing.
    /// Any custom message will be ignored if enabled.
    #[arg(long = "dummyData")]
    pub use_dummy_data: bool,
}

#[derive(Serialize, Debug)]
struct DummyData {
    id: usize,
    a_float: f32,
    an_int_array: Vec<usize>,
    a_string: String,
}

pub fn send(options: &SendOptions, tether_agent: &TetherAgent) -> anyhow::Result<()> {
    info!("Tether Send Utility");

    let (role, id) = tether_agent.description();

    let publish_topic = match &options.plug_topic {
        Some(override_topic) => {
            warn!(
                "Using override topic \"{}\"; agent role, agent ID and plug name will be ignored",
                override_topic
            );
            String::from(override_topic)
        }
        None => {
            let auto_generated_topic = build_topic(role, id, &options.plug_name);
            info!("Using auto-generated topic \"{}\"", &auto_generated_topic);
            auto_generated_topic
        }
    };

    let output = PlugOptionsBuilder::create_output(&options.plug_name)
        .topic(&publish_topic)
        .build(tether_agent)
        .expect("failed to create output plug");

    if options.use_dummy_data {
        let payload = DummyData {
            id: 0,
            a_float: 42.0,
            an_int_array: vec![1, 2, 3, 4],
            a_string: "hello world".into(),
        };
        info!("Sending dummy data {:?}", payload);
        return match tether_agent.encode_and_publish(&output, &payload) {
            Ok(_) => {
                info!("Sent dummy data message OK");
                Ok(())
            }
            Err(e) => Err(e),
        };
    }

    match &options.message_payload_json {
        Some(custom_message) => {
            debug!(
                "Attempting to decode provided custom message \"{}\"",
                &custom_message
            );
            match serde_json::from_str::<serde_json::Value>(custom_message) {
                Ok(encoded) => {
                    let payload =
                        rmp_serde::to_vec_named(&encoded).expect("failed to encode msgpack");
                    tether_agent
                        .publish(&output, Some(&payload))
                        .expect("failed to publish");
                    info!("Sent message OK");
                    Ok(())
                }
                Err(e) => {
                    error!("Could not serialise String -> JSON; error: {}", e);
                    Err(e.into())
                }
            }
        }
        None => {
            warn!("Sending empty message");
            match tether_agent.publish(&output, None) {
                Ok(_) => {
                    info!("Sent empty message OK");
                    Ok(())
                }
                Err(e) => {
                    error!("Failed to send empty message: {}", e);
                    Err(e.into())
                }
            }
        }
    }
}
