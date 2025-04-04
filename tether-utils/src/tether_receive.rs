use clap::Args;
use log::{debug, error, info, warn};
use tether_agent::{three_part_topic::TetherOrCustomTopic, PlugOptionsBuilder, TetherAgent};

#[derive(Args, Default)]
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

pub fn receive(
    options: &ReceiveOptions,
    tether_agent: &mut TetherAgent,
    on_message: fn(plug_name: String, topic: String, decoded: Option<String>),
) {
    info!("Tether Receive Utility");

    let input_def = build_receiver_plug(options);

    let input = input_def
        .build(tether_agent)
        .expect("failed to create input plug");

    info!("Subscribed to topic \"{}\" ...", input.generated_topic());

    loop {
        let mut did_work = false;
        while let Some((topic, payload)) = tether_agent.check_messages() {
            did_work = true;
            let full_topic_string = topic.full_topic_string();
            debug!("Received message on topic \"{}\"", &full_topic_string);
            let plug_name = match topic {
                TetherOrCustomTopic::Custom(_) => String::from("unknown"),
                TetherOrCustomTopic::Tether(tpt) => String::from(tpt.plug_name()),
            };

            if payload.is_empty() {
                debug!("Empty message payload");
                on_message(plug_name, full_topic_string, None);
            } else if let Ok(value) = rmp_serde::from_slice::<rmpv::Value>(&payload) {
                let json = serde_json::to_string(&value).expect("failed to stringify JSON");
                debug!("Decoded MessagePack payload: {}", json);
                on_message(plug_name, full_topic_string, Some(json));
            } else {
                debug!("Failed to decode MessagePack payload");
                if let Ok(s) = String::from_utf8(payload.to_vec()) {
                    warn!("String representation of payload: \"{}\"", s);
                } else {
                    error!("Could not decode payload bytes as string, either");
                }
                on_message(plug_name, full_topic_string, None);
            }
        }
        if !did_work {
            std::thread::sleep(std::time::Duration::from_micros(100)); //0.1 ms
        }
    }
}

fn build_receiver_plug(options: &ReceiveOptions) -> PlugOptionsBuilder {
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
                    provided_name
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
}

#[cfg(test)]
mod tests {
    use tether_agent::TetherAgentOptionsBuilder;

    use crate::tether_receive::build_receiver_plug;

    use super::ReceiveOptions;

    #[test]
    fn default_options() {
        let mut tether_agent = TetherAgentOptionsBuilder::new("tester")
            .build()
            .expect("sorry, these tests require working localhost Broker");

        let options = ReceiveOptions::default();

        let receive_plug = build_receiver_plug(&options)
            .build(&mut tether_agent)
            .expect("build failed");

        assert_eq!(receive_plug.name(), "custom");
        assert_eq!(receive_plug.generated_topic(), "#");
    }

    #[test]
    fn only_topic_custom() {
        let mut tether_agent = TetherAgentOptionsBuilder::new("tester")
            .build()
            .expect("sorry, these tests require working localhost Broker");

        let options = ReceiveOptions {
            subscribe_role: None,
            subscribe_id: None,
            subscribe_plug_name: None,
            subscribe_topic: Some("some/special/plug".into()),
        };

        let receive_plug = build_receiver_plug(&options)
            .build(&mut tether_agent)
            .expect("build failed");

        assert_eq!(receive_plug.name(), "custom");
        assert_eq!(receive_plug.generated_topic(), "some/special/plug");
    }

    #[test]
    fn only_plug_name() {
        let mut tether_agent = TetherAgentOptionsBuilder::new("tester")
            .build()
            .expect("sorry, these tests require working localhost Broker");

        let options = ReceiveOptions {
            subscribe_role: None,
            subscribe_id: None,
            subscribe_plug_name: Some("something".into()),
            subscribe_topic: None,
        };

        let receive_plug = build_receiver_plug(&options)
            .build(&mut tether_agent)
            .expect("build failed");

        assert_eq!(receive_plug.name(), "something");
        assert_eq!(receive_plug.generated_topic(), "+/+/something");
    }

    #[test]
    fn only_role() {
        let mut tether_agent = TetherAgentOptionsBuilder::new("tester")
            .build()
            .expect("sorry, these tests require working localhost Broker");

        let options = ReceiveOptions {
            subscribe_role: Some("something".into()),
            subscribe_id: None,
            subscribe_plug_name: None,
            subscribe_topic: None,
        };

        let receive_plug = build_receiver_plug(&options)
            .build(&mut tether_agent)
            .expect("build failed");

        assert_eq!(receive_plug.name(), "any");
        assert_eq!(receive_plug.generated_topic(), "something/+/+");
    }

    #[test]
    fn only_id() {
        let mut tether_agent = TetherAgentOptionsBuilder::new("tester")
            .build()
            .expect("sorry, these tests require working localhost Broker");

        let options = ReceiveOptions {
            subscribe_role: None,
            subscribe_id: Some("something".into()),
            subscribe_plug_name: None,
            subscribe_topic: None,
        };

        let receive_plug = build_receiver_plug(&options)
            .build(&mut tether_agent)
            .expect("build failed");

        assert_eq!(receive_plug.name(), "any");
        assert_eq!(receive_plug.generated_topic(), "+/something/+");
    }

    #[test]
    fn role_and_id() {
        let mut tether_agent = TetherAgentOptionsBuilder::new("tester")
            .build()
            .expect("sorry, these tests require working localhost Broker");

        let options = ReceiveOptions {
            subscribe_role: Some("x".into()),
            subscribe_id: Some("y".into()),
            subscribe_plug_name: None,
            subscribe_topic: None,
        };

        let receive_plug = build_receiver_plug(&options)
            .build(&mut tether_agent)
            .expect("build failed");

        assert_eq!(receive_plug.name(), "any");
        assert_eq!(receive_plug.generated_topic(), "x/y/+");
    }

    #[test]
    fn role_and_plug_name() {
        let mut tether_agent = TetherAgentOptionsBuilder::new("tester")
            .build()
            .expect("sorry, these tests require working localhost Broker");

        let options = ReceiveOptions {
            subscribe_role: Some("x".into()),
            subscribe_id: None,
            subscribe_plug_name: Some("z".into()),
            subscribe_topic: None,
        };

        let receive_plug = build_receiver_plug(&options)
            .build(&mut tether_agent)
            .expect("build failed");

        assert_eq!(receive_plug.name(), "z");
        assert_eq!(receive_plug.generated_topic(), "x/+/z");
    }

    #[test]
    fn spec_all_three() {
        let mut tether_agent = TetherAgentOptionsBuilder::new("tester")
            .build()
            .expect("sorry, these tests require working localhost Broker");

        let options = ReceiveOptions {
            subscribe_role: Some("x".into()),
            subscribe_id: Some("y".into()),
            subscribe_plug_name: Some("z".into()),
            subscribe_topic: None,
        };

        let receive_plug = build_receiver_plug(&options)
            .build(&mut tether_agent)
            .expect("build failed");

        assert_eq!(receive_plug.name(), "z");
        assert_eq!(receive_plug.generated_topic(), "x/y/z");
    }

    #[test]
    fn redundant_but_valid() {
        let mut tether_agent = TetherAgentOptionsBuilder::new("tester")
            .build()
            .expect("sorry, these tests require working localhost Broker");

        let options = ReceiveOptions {
            subscribe_role: None,
            subscribe_id: None,
            subscribe_plug_name: Some("+".into()),
            subscribe_topic: None,
        };

        let receive_plug = build_receiver_plug(&options)
            .build(&mut tether_agent)
            .expect("build failed");

        assert_eq!(receive_plug.name(), "any");
        assert_eq!(receive_plug.generated_topic(), "+/+/+");
    }
}
