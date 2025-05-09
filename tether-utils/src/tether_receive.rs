use clap::Args;
use log::{debug, error, info, warn};
use tether_agent::{
    ChannelDef, ChannelDefBuilder, ChannelReceiverDef, ChannelReceiverDefBuilder, TetherAgent,
    tether_compliant_topic::TetherOrCustomTopic,
};

#[derive(Args, Default)]
pub struct ReceiveOptions {
    /// Specify a ROLE (instead of wildcard +)
    #[arg(long = "channel.role")]
    pub subscribe_role: Option<String>,

    /// Specify an ID (instead of wildcard #)
    #[arg(long = "channel.id")]
    pub subscribe_id: Option<String>,

    /// Specify a CHANNEL NAME part for the topic (instead of wildcard +)
    #[arg(long = "channel.name")]
    pub subscribe_channel_name: Option<String>,

    /// Override topic to subscribe; setting this will
    /// ignore any `channel.` options you may have set, since the
    /// topic is built manually.
    #[arg(long = "topic")]
    pub subscribe_topic: Option<String>,
}

pub fn receive(
    options: &ReceiveOptions,
    tether_agent: &mut TetherAgent,
    on_message: fn(channel_name: String, topic: String, decoded: Option<String>),
) {
    info!("Tether Receive Utility");

    let channel_def = build_receiver(options, tether_agent);

    // let channel = channel_options
    //     .build(tether_agent)
    //     .expect("failed to create Channel Receiver");

    // let channel = tether_agent
    //     .create_receiver_with_def(channel_def)
    //     .expect("failed to create Receiver");

    if let Some(client) = tether_agent.client_mut() {
        client
            .subscribe(channel_def.generated_topic(), channel_def.qos())
            .expect("failed to subscribe");

        info!(
            "Subscribed to topic \"{}\" ...",
            channel_def.generated_topic()
        );

        loop {
            let mut did_work = false;
            while let Some((topic, payload)) = tether_agent.check_messages() {
                did_work = true;
                let full_topic_string = topic.full_topic_string();
                debug!("Received message on topic \"{}\"", &full_topic_string);
                let channel_name = match topic {
                    TetherOrCustomTopic::Custom(_) => String::from("unknown"),
                    TetherOrCustomTopic::Tether(tpt) => String::from(tpt.channel_name()),
                };

                if payload.is_empty() {
                    debug!("Empty message payload");
                    on_message(channel_name, full_topic_string, None);
                } else if let Ok(value) = rmp_serde::from_slice::<rmpv::Value>(&payload) {
                    let json = serde_json::to_string(&value).expect("failed to stringify JSON");
                    debug!("Decoded MessagePack payload: {}", json);
                    on_message(channel_name, full_topic_string, Some(json));
                } else {
                    debug!("Failed to decode MessagePack payload");
                    if let Ok(s) = String::from_utf8(payload.to_vec()) {
                        warn!("String representation of payload: \"{}\"", s);
                    } else {
                        error!("Could not decode payload bytes as string, either");
                    }
                    on_message(channel_name, full_topic_string, None);
                }
            }
            if !did_work {
                std::thread::sleep(std::time::Duration::from_micros(100)); //0.1 ms
            }
        }
    } else {
        error!("Failed to subscribe via Client!");
    }
}

fn build_receiver(options: &ReceiveOptions, tether_agent: &TetherAgent) -> ChannelReceiverDef {
    if options.subscribe_id.is_some()
        || options.subscribe_role.is_some()
        || options.subscribe_channel_name.is_some()
    {
        debug!(
            "TPT Overrides apply: {:?}, {:?}, {:?}",
            &options.subscribe_id, &options.subscribe_role, &options.subscribe_channel_name
        );
        ChannelReceiverDefBuilder::new(match &options.subscribe_channel_name {
            Some(provided_name) => provided_name,
            None => "+",
        })
        .role(options.subscribe_role.as_deref())
        .id(options.subscribe_id.as_deref())
        .build(tether_agent.config())
    } else {
        debug!(
            "Using custom override topic \"{:?}\"",
            &options.subscribe_topic
        );
        ChannelReceiverDefBuilder::new("custom")
            .override_topic(Some(options.subscribe_topic.as_deref().unwrap_or("#")))
            .build(tether_agent.config())
    }
}

#[cfg(test)]
mod tests {
    use tether_agent::{ChannelDef, TetherAgentBuilder};

    use crate::tether_receive::build_receiver;

    use super::ReceiveOptions;

    #[test]
    fn default_options() {
        let tether_agent = TetherAgentBuilder::new("tester")
            .auto_connect(false)
            .build()
            .unwrap();

        let options = ReceiveOptions::default();

        let receiver_def = build_receiver(&options, &tether_agent);

        assert_eq!(receiver_def.name(), "custom");
        assert_eq!(receiver_def.generated_topic(), "#");
    }

    #[test]
    fn only_topic_custom() {
        let tether_agent = TetherAgentBuilder::new("tester")
            .auto_connect(false)
            .build()
            .unwrap();

        let options = ReceiveOptions {
            subscribe_role: None,
            subscribe_id: None,
            subscribe_channel_name: None,
            subscribe_topic: Some("some/channel/special/fourpart".into()),
        };

        let receiver_def = build_receiver(&options, &tether_agent);

        assert_eq!(receiver_def.name(), "custom");
        assert_eq!(
            receiver_def.generated_topic(),
            "some/channel/special/fourpart"
        );
    }

    #[test]
    fn only_chanel_name() {
        let tether_agent = TetherAgentBuilder::new("tester")
            .auto_connect(false)
            .build()
            .unwrap();

        let options = ReceiveOptions {
            subscribe_role: None,
            subscribe_id: None,
            subscribe_channel_name: Some("something".into()),
            subscribe_topic: None,
        };

        let receiver_def = build_receiver(&options, &tether_agent);

        assert_eq!(receiver_def.name(), "something");
        assert_eq!(receiver_def.generated_topic(), "+/something/#");
    }

    #[test]
    fn only_role() {
        let tether_agent = TetherAgentBuilder::new("tester")
            .auto_connect(false)
            .build()
            .unwrap();

        let options = ReceiveOptions {
            subscribe_role: Some("something".into()),
            subscribe_id: None,
            subscribe_channel_name: None,
            subscribe_topic: None,
        };

        let receiver_def = build_receiver(&options, &tether_agent);

        assert_eq!(receiver_def.name(), "+");
        assert_eq!(receiver_def.generated_topic(), "something/+/#");
    }

    #[test]
    fn only_id() {
        let tether_agent = TetherAgentBuilder::new("tester")
            .auto_connect(false)
            .build()
            .unwrap();

        let options = ReceiveOptions {
            subscribe_role: None,
            subscribe_id: Some("something".into()),
            subscribe_channel_name: None,
            subscribe_topic: None,
        };

        let receiver_def = build_receiver(&options, &tether_agent);

        assert_eq!(receiver_def.name(), "+");
        assert_eq!(receiver_def.generated_topic(), "+/+/something");
    }

    #[test]
    fn role_and_id() {
        let tether_agent = TetherAgentBuilder::new("tester")
            .auto_connect(false)
            .build()
            .unwrap();

        let options = ReceiveOptions {
            subscribe_role: Some("x".into()),
            subscribe_id: Some("y".into()),
            subscribe_channel_name: None,
            subscribe_topic: None,
        };

        let receiver_def = build_receiver(&options, &tether_agent);

        assert_eq!(receiver_def.name(), "+");
        assert_eq!(receiver_def.generated_topic(), "x/+/y");
    }

    #[test]
    fn role_and_channel_name() {
        let tether_agent = TetherAgentBuilder::new("tester")
            .auto_connect(false)
            .build()
            .unwrap();

        let options = ReceiveOptions {
            subscribe_role: Some("x".into()),
            subscribe_id: None,
            subscribe_channel_name: Some("z".into()),
            subscribe_topic: None,
        };

        let receiver_def = build_receiver(&options, &tether_agent);

        assert_eq!(receiver_def.name(), "z");
        assert_eq!(receiver_def.generated_topic(), "x/z/#");
    }

    #[test]
    fn spec_all_three() {
        let tether_agent = TetherAgentBuilder::new("tester")
            .auto_connect(false)
            .build()
            .unwrap();

        let options = ReceiveOptions {
            subscribe_role: Some("x".into()),
            subscribe_channel_name: Some("z".into()),
            subscribe_id: Some("y".into()),
            subscribe_topic: None,
        };

        let receiver_def = build_receiver(&options, &tether_agent);

        assert_eq!(receiver_def.name(), "z");
        assert_eq!(receiver_def.generated_topic(), "x/z/y");
    }

    #[test]
    fn redundant_but_valid() {
        let tether_agent = TetherAgentBuilder::new("tester")
            .auto_connect(false)
            .build()
            .unwrap();

        let options = ReceiveOptions {
            subscribe_role: None,
            subscribe_id: None,
            subscribe_channel_name: Some("+".into()),
            subscribe_topic: None,
        };

        let receiver_def = build_receiver(&options, &tether_agent);

        assert_eq!(receiver_def.name(), "+");
        assert_eq!(receiver_def.generated_topic(), "+/+/#");
    }
}
