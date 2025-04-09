use anyhow::anyhow;
use log::{debug, error, info, warn};

use crate::{
    definitions::ChannelDefinitionCommon, tether_compliant_topic::TetherCompliantTopic,
    ChannelDefinition, TetherAgent,
};

use super::{
    tether_compliant_topic::TetherOrCustomTopic, ChannelReceiverDefinition, ChannelSenderDefinition,
};

pub struct ChannelReceiverOptions {
    channel_name: String,
    qos: Option<i32>,
    override_subscribe_role: Option<String>,
    override_subscribe_id: Option<String>,
    override_subscribe_channel_name: Option<String>,
    override_topic: Option<String>,
}

pub struct ChannelSenderOptions {
    channel_name: String,
    qos: Option<i32>,
    override_publish_role: Option<String>,
    override_publish_id: Option<String>,
    override_topic: Option<String>,
    retain: Option<bool>,
}

/// This is the definition of a Channel Receiver or Sender.
///
/// You typically don't use an instance of this directly; call `.build()` at the
/// end of the chain to get a usable **ChannelDefinition**
pub enum ChannelOptionsBuilder {
    ChannelReceiver(ChannelReceiverOptions),
    ChannelSender(ChannelSenderOptions),
}

impl ChannelOptionsBuilder {
    pub fn create_receiver(name: &str) -> ChannelOptionsBuilder {
        ChannelOptionsBuilder::ChannelReceiver(ChannelReceiverOptions {
            channel_name: String::from(name),
            override_subscribe_id: None,
            override_subscribe_role: None,
            override_subscribe_channel_name: None,
            override_topic: None,
            qos: None,
        })
    }

    pub fn create_sender(name: &str) -> ChannelOptionsBuilder {
        ChannelOptionsBuilder::ChannelSender(ChannelSenderOptions {
            channel_name: String::from(name),
            override_publish_id: None,
            override_publish_role: None,
            override_topic: None,
            qos: None,
            retain: None,
        })
    }

    pub fn qos(mut self, qos: Option<i32>) -> Self {
        match &mut self {
            ChannelOptionsBuilder::ChannelReceiver(s) => s.qos = qos,
            ChannelOptionsBuilder::ChannelSender(s) => s.qos = qos,
        };
        self
    }

    /**
    Override the "role" part of the topic that gets generated for this Channel.
    - For Channel Receivers, this means you want to be specific about the Role part
      of the topic, instead of using the default wildcard `+` at this location
    - For Channel Senders, this means you want to override the Role part instead
      of using your Agent's "own" Role with which you created the Tether Agent

    If you override the entire topic using `.topic` this will be ignored.
    */
    pub fn role(mut self, role: Option<&str>) -> Self {
        match &mut self {
            ChannelOptionsBuilder::ChannelReceiver(s) => {
                if s.override_topic.is_some() {
                    error!("Override topic was also provided; this will take precedence");
                } else {
                    s.override_subscribe_role = role.map(|s| s.into());
                }
            }
            ChannelOptionsBuilder::ChannelSender(s) => {
                if s.override_topic.is_some() {
                    error!("Override topic was also provided; this will take precedence");
                } else {
                    s.override_publish_role = role.map(|s| s.into());
                }
            }
        };
        self
    }

    /**
    Override the "id" part of the topic that gets generated for this Channel.
    - For Channel Receivers, this means you want to be specific about the ID part
      of the topic, instead of using the default wildcard `+` at this location
    - For Channel Senders, this means you want to override the ID part instead
      of using your Agent's "own" ID which you specified (or left blank, i.e. "any")
      when creating the Tether Agent

    If you override the entire topic using `.topic` this will be ignored.
    */
    pub fn id(mut self, id: Option<&str>) -> Self {
        match &mut self {
            ChannelOptionsBuilder::ChannelReceiver(s) => {
                if s.override_topic.is_some() {
                    error!("Override topic was also provided; this will take precedence");
                } else {
                    s.override_subscribe_id = id.map(|s| s.into());
                }
            }
            ChannelOptionsBuilder::ChannelSender(s) => {
                if s.override_topic.is_some() {
                    error!("Override topic was also provided; this will take precedence");
                } else {
                    s.override_publish_id = id.map(|s| s.into());
                }
            }
        };
        self
    }

    /// Override the "name" part of the topic that gets generated for this Channel.
    /// This is mainly to facilitate wildcard subscriptions such as
    /// `someRole/+` instead of `someRole/originalChannelName`.
    ///
    /// In the case of Receiver Topics, a wildcard `+` can be used to substitute
    /// the last part of the topic as in `role/id/+`
    ///
    /// Channel Senders will ignore (with an error) any attempt to change the name after
    /// instantiation.
    pub fn name(mut self, override_channel_name: Option<&str>) -> Self {
        match &mut self {
            ChannelOptionsBuilder::ChannelReceiver(opt) => {
                if opt.override_topic.is_some() {
                    error!("Override topic was also provided; this will take precedence");
                }
                if override_channel_name.is_some() {
                    opt.override_subscribe_channel_name = override_channel_name.map(|s| s.into());
                } else {
                    debug!("Override Channel name set to None; will use original name \"{}\" given in ::create_receiver constructor", opt.channel_name);
                }
            }
            ChannelOptionsBuilder::ChannelSender(_) => {
                error!(
                    "Channel Senders cannot change their name part after ::create_sender constructor"
                );
            }
        };
        self
    }

    /// Call this if you would like your Channel Receiver to match **any channel**.
    /// This is equivalent to `.name(Some("+"))` but is provided for convenience
    /// since it does not require you to remember the wildcard string.
    ///
    /// This also does not prevent you from further restricting the topic
    /// subscription match by Role and/or ID. So, for example, if you are
    /// interested in **all messages** from an Agent with the role `"brain"`,
    /// it is valid to create a channel with `.role("brain").any_channel()` and this
    /// will subscribe to `"brain/+/#"` as expected.
    pub fn any_channel(mut self) -> Self {
        match &mut self {
            ChannelOptionsBuilder::ChannelReceiver(opt) => {
                opt.override_subscribe_channel_name = Some("+".into());
            }
            ChannelOptionsBuilder::ChannelSender(_) => {
                error!(
                    "Channel Senders cannot change their name part after ::create_sender constructor"
                );
            }
        }
        self
    }

    /// Override the final topic to use for publishing or subscribing. The provided topic **will** be checked
    /// against the Tether Compliant Topic (TCT) convention, but the function **will not** reject topic strings - just
    /// produce a warning. It's therefore valid to use a wildcard such as "#", for Receivers (subscribing).
    ///
    /// Any customisations specified using `.role(...)` or `.id(...)` will be ignored if this function is called
    /// after these.
    ///
    /// By default, the override_topic is None, but you can specify None explicitly using this function.
    pub fn topic(mut self, override_topic: Option<&str>) -> Self {
        match override_topic {
            Some(t) => {
                if TryInto::<TetherCompliantTopic>::try_into(t).is_ok() {
                    info!("Custom topic passes Tether Compliant Topic validation");
                } else if t == "#" {
                    info!("Wildcard \"#\" custom topics are not Tether Compliant Topics but are valid");
                } else {
                    warn!(
                        "Could not convert \"{}\" into Tether Compliant Topic; presumably you know what you're doing!",
                        t
                    );
                }
                match &mut self {
                    ChannelOptionsBuilder::ChannelReceiver(s) => s.override_topic = Some(t.into()),
                    ChannelOptionsBuilder::ChannelSender(s) => s.override_topic = Some(t.into()),
                };
            }
            None => {
                match &mut self {
                    ChannelOptionsBuilder::ChannelReceiver(s) => s.override_topic = None,
                    ChannelOptionsBuilder::ChannelSender(s) => s.override_topic = None,
                };
            }
        }
        self
    }

    pub fn retain(mut self, should_retain: Option<bool>) -> Self {
        match &mut self {
            Self::ChannelReceiver(_) => {
                error!("Cannot set retain flag on Receiver / subscription");
            }
            Self::ChannelSender(s) => {
                s.retain = should_retain;
            }
        }
        self
    }

    /// Finalise the options (substituting suitable defaults if no custom values have been
    /// provided) and return a valid ChannelDefinition that you can actually use.
    pub fn build(self, tether_agent: &mut TetherAgent) -> anyhow::Result<ChannelDefinition> {
        match self {
            Self::ChannelReceiver(channel_options) => {
                let tpt: TetherOrCustomTopic = match channel_options.override_topic {
                    Some(custom) => TetherOrCustomTopic::Custom(custom),
                    None => {
                        debug!("Not a custom topic; provided overrides: role = {:?}, id = {:?}, name = {:?}", channel_options.override_subscribe_role, channel_options.override_subscribe_id, channel_options.override_subscribe_channel_name);

                        TetherOrCustomTopic::Tether(TetherCompliantTopic::new_for_subscribe(
                            &channel_options
                                .override_subscribe_channel_name
                                .unwrap_or(channel_options.channel_name.clone()),
                            channel_options.override_subscribe_role.as_deref(),
                            channel_options.override_subscribe_id.as_deref(),
                        ))
                    }
                };
                let channel_definition = ChannelReceiverDefinition::new(
                    &channel_options.channel_name,
                    tpt,
                    channel_options.qos,
                );

                // This is really only useful for testing purposes.
                if !tether_agent.auto_connect_enabled() {
                    warn!("Auto-connect is disabled, skipping subscription");
                    return Ok(ChannelDefinition::ChannelReceiver(channel_definition));
                }

                if let Some(client) = &tether_agent.client {
                    match client.subscribe(
                        channel_definition.generated_topic(),
                        match channel_definition.qos() {
                            0 => rumqttc::QoS::AtMostOnce,
                            1 => rumqttc::QoS::AtLeastOnce,
                            2 => rumqttc::QoS::ExactlyOnce,
                            _ => rumqttc::QoS::AtLeastOnce,
                        },
                    ) {
                        Ok(res) => {
                            debug!(
                                "This topic was fine: \"{}\"",
                                channel_definition.generated_topic()
                            );
                            debug!("Server respond OK for subscribe: {res:?}");
                            Ok(ChannelDefinition::ChannelReceiver(channel_definition))
                        }
                        Err(_e) => Err(anyhow!("ClientError")),
                    }
                } else {
                    Err(anyhow!("Client not available for subscription"))
                }
            }
            Self::ChannelSender(channel_options) => {
                let tpt: TetherOrCustomTopic = match channel_options.override_topic {
                    Some(custom) => {
                        warn!(
                            "Custom topic override: \"{}\" - all other options ignored",
                            custom
                        );
                        TetherOrCustomTopic::Custom(custom)
                    }
                    None => {
                        let optional_id_part = match channel_options.override_publish_id {
                            Some(id) => {
                                debug!("Publish ID was overriden at Channel options level. The Agent ID will be ignored.");
                                Some(id)
                            }
                            None => {
                                debug!("Publish ID was not overriden at Channel options level. The Agent ID will be used instead, if specified in Agent creation.");
                                tether_agent.id().map(String::from)
                            }
                        };

                        TetherOrCustomTopic::Tether(TetherCompliantTopic::new_for_publish(
                            tether_agent,
                            &channel_options.channel_name,
                            channel_options.override_publish_role.as_deref(),
                            optional_id_part.as_deref(),
                        ))
                    }
                };

                let channel_definition = ChannelSenderDefinition::new(
                    &channel_options.channel_name,
                    tpt,
                    channel_options.qos,
                    channel_options.retain,
                );
                Ok(ChannelDefinition::ChannelSender(channel_definition))
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::{ChannelOptionsBuilder, TetherAgentOptionsBuilder};

    // fn verbose_logging() {
    //     use env_logger::{Builder, Env};
    //     let mut logger_builder = Builder::from_env(Env::default().default_filter_or("debug"));
    //     logger_builder.init();
    // }

    #[test]
    fn default_receiver_channel() {
        // verbose_logging();
        let mut tether_agent = TetherAgentOptionsBuilder::new("tester")
            .auto_connect(false)
            .build()
            .expect("sorry, these tests require working localhost Broker");
        let receiver = ChannelOptionsBuilder::create_receiver("one")
            .build(&mut tether_agent)
            .unwrap();
        assert_eq!(receiver.name(), "one");
        assert_eq!(receiver.generated_topic(), "+/one/#");
    }

    #[test]
    /// This is a fairly trivial example, but contrast with the test
    /// `sender_channel_default_but_agent_id_custom`: although a custom ID was set for the
    /// Agent, this does not affect the Topic for a Channel Receiver created without any
    /// explicit overrides.
    fn default_channel_receiver_with_agent_custom_id() {
        // verbose_logging();
        let mut tether_agent = TetherAgentOptionsBuilder::new("tester")
            .auto_connect(false)
            .id(Some("verySpecialGroup"))
            .build()
            .expect("sorry, these tests require working localhost Broker");
        let receiver = ChannelOptionsBuilder::create_receiver("one")
            .build(&mut tether_agent)
            .unwrap();
        assert_eq!(receiver.name(), "one");
        assert_eq!(receiver.generated_topic(), "+/one/#");
    }

    #[test]
    fn default_channel_sender() {
        let mut tether_agent = TetherAgentOptionsBuilder::new("tester")
            .auto_connect(false)
            .build()
            .expect("sorry, these tests require working localhost Broker");
        let channel = ChannelOptionsBuilder::create_sender("two")
            .build(&mut tether_agent)
            .unwrap();
        assert_eq!(channel.name(), "two");
        assert_eq!(channel.generated_topic(), "tester/two");
    }

    #[test]
    /// This is identical to the case in which a Channel Sender is created with defaults (no overrides),
    /// BUT the Agent had a custom ID set, which means that the final topic includes this custom
    /// ID/Group value.
    fn sender_channel_default_but_agent_id_custom() {
        let mut tether_agent = TetherAgentOptionsBuilder::new("tester")
            .auto_connect(false)
            .id(Some("specialCustomGrouping"))
            .build()
            .expect("sorry, these tests require working localhost Broker");
        let channel = ChannelOptionsBuilder::create_sender("somethingStandard")
            .build(&mut tether_agent)
            .unwrap();
        assert_eq!(channel.name(), "somethingStandard");
        assert_eq!(
            channel.generated_topic(),
            "tester/somethingStandard/specialCustomGrouping"
        );
    }

    #[test]
    fn receiver_id_andor_role() {
        let mut tether_agent = TetherAgentOptionsBuilder::new("tester")
            .auto_connect(false)
            .build()
            .expect("sorry, these tests require working localhost Broker");

        let receive_role_only = ChannelOptionsBuilder::create_receiver("theChannel")
            .role(Some("specificRole"))
            .build(&mut tether_agent)
            .unwrap();
        assert_eq!(receive_role_only.name(), "theChannel");
        assert_eq!(
            receive_role_only.generated_topic(),
            "specificRole/theChannel/#"
        );

        let receiver_id_only = ChannelOptionsBuilder::create_receiver("theChannel")
            .id(Some("specificID"))
            .build(&mut tether_agent)
            .unwrap();
        assert_eq!(receiver_id_only.name(), "theChannel");
        assert_eq!(
            receiver_id_only.generated_topic(),
            "+/theChannel/specificID"
        );

        let receiver_both_custom = ChannelOptionsBuilder::create_receiver("theChannel")
            .id(Some("specificID"))
            .role(Some("specificRole"))
            .build(&mut tether_agent)
            .unwrap();
        assert_eq!(receiver_both_custom.name(), "theChannel");
        assert_eq!(
            receiver_both_custom.generated_topic(),
            "specificRole/theChannel/specificID"
        );
    }

    #[test]
    /// If the end-user implicitly specifies the chanel name part (does not set it to Some(_)
    /// or None) then the ID and/or Role parts will change but the Channel Name part will
    /// remain the "original" / default
    /// Contrast with receiver_specific_id_andor_role_no_chanel_name below.
    fn receiver_specific_id_andor_role_with_channel_name() {
        let mut tether_agent = TetherAgentOptionsBuilder::new("tester")
            .auto_connect(false)
            .build()
            .expect("sorry, these tests require working localhost Broker");

        let receiver_role_only = ChannelOptionsBuilder::create_receiver("theChannel")
            .role(Some("specificRole"))
            .build(&mut tether_agent)
            .unwrap();
        assert_eq!(receiver_role_only.name(), "theChannel");
        assert_eq!(
            receiver_role_only.generated_topic(),
            "specificRole/theChannel/#"
        );

        let receiver_id_only = ChannelOptionsBuilder::create_receiver("theChannel")
            .id(Some("specificID"))
            .build(&mut tether_agent)
            .unwrap();
        assert_eq!(receiver_id_only.name(), "theChannel");
        assert_eq!(
            receiver_id_only.generated_topic(),
            "+/theChannel/specificID"
        );

        let receiver_both = ChannelOptionsBuilder::create_receiver("theChannel")
            .id(Some("specificID"))
            .role(Some("specificRole"))
            .build(&mut tether_agent)
            .unwrap();
        assert_eq!(receiver_both.name(), "theChannel");
        assert_eq!(
            receiver_both.generated_topic(),
            "specificRole/theChannel/specificID"
        );
    }

    #[test]
    /// Unlike receiver_specific_id_andor_role_with_channel_name, this tests the situation where
    /// the end-user (possibly) specifies the ID and/or Role, but also explicitly
    /// sets the Channel Name to Some("+"), ie. "use a wildcard at this
    /// position instead" - and NOT the original channel name.
    fn receiver_specific_id_andor_role_no_channel_name() {
        let mut tether_agent = TetherAgentOptionsBuilder::new("tester")
            .auto_connect(false)
            .build()
            .expect("sorry, these tests require working localhost Broker");

        let receiver_only_chanel_name_none = ChannelOptionsBuilder::create_receiver("theChannel")
            .name(Some("+"))
            .build(&mut tether_agent)
            .unwrap();
        assert_eq!(receiver_only_chanel_name_none.name(), "theChannel");
        assert_eq!(receiver_only_chanel_name_none.generated_topic(), "+/+/#");

        let receiver_role_only = ChannelOptionsBuilder::create_receiver("theChannel")
            .name(Some("+"))
            .role(Some("specificRole"))
            .build(&mut tether_agent)
            .unwrap();
        assert_eq!(receiver_role_only.name(), "theChannel");
        assert_eq!(receiver_role_only.generated_topic(), "specificRole/+/#");

        let receiver_id_only = ChannelOptionsBuilder::create_receiver("theChannel")
            // .name(Some("+"))
            .any_channel() // equivalent to Some("+")
            .id(Some("specificID"))
            .build(&mut tether_agent)
            .unwrap();
        assert_eq!(receiver_id_only.name(), "theChannel");
        assert_eq!(receiver_id_only.generated_topic(), "+/+/specificID");

        let receiver_both = ChannelOptionsBuilder::create_receiver("theChannel")
            .name(Some("+"))
            .id(Some("specificID"))
            .role(Some("specificRole"))
            .build(&mut tether_agent)
            .unwrap();
        assert_eq!(receiver_both.name(), "theChannel");
        assert_eq!(receiver_both.generated_topic(), "specificRole/+/specificID");
    }

    #[test]
    fn any_name_but_specify_role() {
        // Some fairly niche cases here

        let mut tether_agent = TetherAgentOptionsBuilder::new("tester")
            .auto_connect(false)
            .build()
            .expect("sorry, these tests require working localhost Broker");

        let receiver_any_channel = ChannelOptionsBuilder::create_receiver("aTest")
            .any_channel()
            .build(&mut tether_agent)
            .unwrap();

        assert_eq!(receiver_any_channel.name(), "aTest");
        assert_eq!(receiver_any_channel.generated_topic(), "+/+/#");

        let receiver_specify_role = ChannelOptionsBuilder::create_receiver("aTest")
            .any_channel()
            .role(Some("brain"))
            .build(&mut tether_agent)
            .unwrap();

        assert_eq!(receiver_specify_role.name(), "aTest");
        assert_eq!(receiver_specify_role.generated_topic(), "brain/+/#");
    }

    #[test]
    fn sender_custom() {
        let mut tether_agent = TetherAgentOptionsBuilder::new("tester")
            .auto_connect(false)
            .build()
            .expect("sorry, these tests require working localhost Broker");

        let sender_custom_role = ChannelOptionsBuilder::create_sender("theChannelSender")
            .role(Some("customRole"))
            .build(&mut tether_agent)
            .unwrap();
        assert_eq!(sender_custom_role.name(), "theChannelSender");
        assert_eq!(
            sender_custom_role.generated_topic(),
            "customRole/theChannelSender"
        );

        let sender_custom_id = ChannelOptionsBuilder::create_sender("theChannelSender")
            .id(Some("customID"))
            .build(&mut tether_agent)
            .unwrap();
        assert_eq!(sender_custom_id.name(), "theChannelSender");
        assert_eq!(
            sender_custom_id.generated_topic(),
            "tester/theChannelSender/customID"
        );

        let sender_custom_both = ChannelOptionsBuilder::create_sender("theChannelSender")
            .role(Some("customRole"))
            .id(Some("customID"))
            .build(&mut tether_agent)
            .unwrap();
        assert_eq!(sender_custom_both.name(), "theChannelSender");
        assert_eq!(
            sender_custom_both.generated_topic(),
            "customRole/theChannelSender/customID"
        );
    }

    #[test]
    fn receiver_manual_topics() {
        let mut tether_agent = TetherAgentOptionsBuilder::new("tester")
            .auto_connect(false)
            .build()
            .expect("sorry, these tests require working localhost Broker");

        let receiver_all = ChannelOptionsBuilder::create_receiver("everything")
            .topic(Some("#"))
            .build(&mut tether_agent)
            .unwrap();
        assert_eq!(receiver_all.name(), "everything");
        assert_eq!(receiver_all.generated_topic(), "#");

        let receiver_nontether = ChannelOptionsBuilder::create_receiver("weird")
            .topic(Some("foo/bar/baz/one/two/three"))
            .build(&mut tether_agent)
            .unwrap();
        assert_eq!(receiver_nontether.name(), "weird");
        assert_eq!(
            receiver_nontether.generated_topic(),
            "foo/bar/baz/one/two/three"
        );
    }
}
