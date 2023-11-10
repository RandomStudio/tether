use clap::Args;
use log::{debug, error, info, warn};
use tether_agent::{mqtt::Message, PlugOptionsBuilder, TetherAgent, TetherOrCustomTopic};

#[derive(Args)]
pub struct ReceiveOptions {
    /// Specify a ROLE (instead of wildcard +)
    #[arg(long = "plug.role")]
    pub subscribe_role: Option<String>,

    /// Specify an ID (instead of wildcard +)
    #[arg(long = "plug.id")]
    pub subscribe_id: Option<String>,

    /// Specify a PLUG NAME part for the topic (instead of wildcard +)
    #[arg(long = "plug.name")]
    pub subscribe_plug_name: Option<String>,

    /// Override topic to subscribe; setting this will
    /// ignore any `plug.` options you may have set, since the
    /// topic is built manually.
    #[arg(long = "topic")]
    pub subscribe_topic: Option<String>,
}

impl Default for ReceiveOptions {
    fn default() -> Self {
        ReceiveOptions {
            subscribe_topic: None,
            subscribe_role: None,
            subscribe_id: None,
            subscribe_plug_name: None,
        }
    }
}

pub fn receive(
    options: &ReceiveOptions,
    tether_agent: &TetherAgent,
    on_message: fn(plug_name: String, message: Message, decoded: Option<String>),
) {
    info!("Tether Receive Utility");

    let input_def = {
        if options.subscribe_id.is_some()
            || options.subscribe_role.is_some()
            || options.subscribe_plug_name.is_some()
        {
            debug!(
                "TPT Overrides apply: {:?}, {:?}, {:?}",
                &options.subscribe_id, &options.subscribe_role, &options.subscribe_plug_name
            );
            PlugOptionsBuilder::create_input(match &options.subscribe_plug_name {
                Some(provided_name) => {
                    if provided_name.as_str() == "+" {
                        "any"
                    } else {
                        &provided_name
                    }
                }
                None => "any",
            })
            .role(options.subscribe_role.as_deref())
            .id(options.subscribe_id.as_deref())
            .name(match &options.subscribe_plug_name {
                Some(provided_name_part) => {
                    if provided_name_part.as_str() == "+" {
                        Some("+")
                    } else {
                        None
                    }
                }
                None => {
                    if options.subscribe_id.is_some() || options.subscribe_role.is_some() {
                        // No plug name part was supplied, but other parts were; therefore
                        // in this case Tether Receive should subscribr to all messages
                        // matching or both of the specified Agent and/or Role
                        Some("+")
                    } else {
                        // No plug name part was supplied, but neither was anything else
                        // Logically, we shouldn't reach this point because of the outer condition
                        // but it must be provided here for completeness
                        None
                    }
                }
            })
        } else {
            debug!(
                "Using custom override topic \"{:?}\"",
                &options.subscribe_topic
            );
            PlugOptionsBuilder::create_input("custom")
                .topic(Some(options.subscribe_topic.as_deref().unwrap_or("#")))
        }
    };

    let input = input_def
        .build(tether_agent)
        .expect("failed to create input plug");

    info!("Subscribed to topic \"{}\" ...", input.topic());

    loop {
        let mut did_work = false;
        while let Some((topic, message)) = tether_agent.check_messages() {
            did_work = true;
            debug!("Received message on topic \"{}\"", message.topic());
            let plug_name = match topic {
                TetherOrCustomTopic::Custom(_) => String::from("unknown"),
                TetherOrCustomTopic::Tether(tpt) => String::from(tpt.plug_name()),
            };

            let bytes = message.payload();
            if bytes.is_empty() {
                debug!("Empty message payload");
                on_message(plug_name, message, None);
            } else if let Ok(value) = rmp_serde::from_slice::<rmpv::Value>(bytes) {
                let json = serde_json::to_string(&value).expect("failed to stringify JSON");
                debug!("Decoded MessagePack payload: {}", json);
                on_message(plug_name, message, Some(json));
            } else {
                debug!("Failed to decode MessagePack payload");
                if let Ok(s) = String::from_utf8(bytes.to_vec()) {
                    warn!("String representation of payload: \"{}\"", s);
                } else {
                    error!("Could not decode payload bytes as string, either");
                }
                on_message(plug_name, message, None);
            }
        }
        if !did_work {
            std::thread::sleep(std::time::Duration::from_micros(100)); //0.1 ms
        }
    }
}
