use clap::Args;
use log::{debug, error, info, warn};
use serde::Serialize;
use tether_agent::{PlugOptionsBuilder, TetherAgent};

#[derive(Args)]
pub struct SendOptions {
    /// Overide the auto-generated topic with your own, to use with every published message
    #[arg(long = "plug.name")]
    pub plug_name: Option<String>,

    /// Overide Tether Agent role with your own, to use with every published message
    #[arg(long = "plug.role")]
    pub plug_role: Option<String>,

    /// Overide Tether Agent ID with your own, to use with every published message
    #[arg(long = "plug.id")]
    pub plug_id: Option<String>,

    /// Overide the entire topic string (ignoring any defaults or customisations applied elsewhere),
    /// to use with every published message
    #[arg(long = "topic")]
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

pub fn send(options: &SendOptions, tether_agent: &mut TetherAgent) -> anyhow::Result<()> {
    info!("Tether Send Utility");

    let plug_name = options.plug_name.clone().unwrap_or("testMessages".into());

    let output = PlugOptionsBuilder::create_output(&plug_name)
        .role(options.plug_role.as_deref())
        .id(options.plug_id.as_deref())
        .topic(options.plug_topic.as_deref())
        .build(tether_agent)
        .expect("failed to create output plug");

    info!("Sending on topic \"{}\" ...", output.topic());

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
                    Err(e)
                }
            }
        }
    }
}
