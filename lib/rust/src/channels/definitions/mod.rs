use super::tether_compliant_topic::TetherOrCustomTopic;

pub mod receiver_builder;
pub mod sender_builder;

pub use receiver_builder::ChannelReceiverBuilder;
pub use sender_builder::ChannelSenderBuilder;

/**
A Channel Builder is used for creating a Channel Definition.
*/
pub trait ChannelBuilder {
    fn new(name: &str) -> Self;
    fn qos(self, qos: Option<i32>) -> Self;
    fn role(self, role: Option<&str>) -> Self;
    fn id(self, id: Option<&str>) -> Self;
    fn override_name(self, override_channel_name: Option<&str>) -> Self;
    fn override_topic(self, override_topic: Option<&str>) -> Self;
}

/**
A Channel Definition is intended to encapsulate only the essential metadata
and configuration needed to describe a Channel. In contrast with a Channel Sender/Receiver,
it is **not** responsible for actually sending or receiving messages on that Channel.
*/
pub trait ChannelDefinition<'a> {
    fn name(&'a self) -> &'a str;
    /// Return the generated topic string actually used by the Channel
    fn generated_topic(&'a self) -> &'a str;
    /// Return the custom or Tether-compliant topic
    fn topic(&'a self) -> &'a TetherOrCustomTopic;
    fn qos(&'a self) -> i32;
}

#[derive(Clone)]
pub struct ChannelSenderDefinition {
    pub name: String,
    pub generated_topic: String,
    pub topic: TetherOrCustomTopic,
    pub qos: i32,
    pub retain: bool,
}

impl ChannelSenderDefinition {
    pub fn retain(&self) -> bool {
        self.retain
    }
}

#[derive(Clone)]
pub struct ChannelReceiverDefinition {
    pub name: String,
    pub generated_topic: String,
    pub topic: TetherOrCustomTopic,
    pub qos: i32,
}

impl<'a> ChannelDefinition<'a> for ChannelSenderDefinition {
    fn name(&'a self) -> &'a str {
        &self.name
    }

    fn generated_topic(&'a self) -> &'a str {
        &self.generated_topic
    }

    fn topic(&'a self) -> &'a TetherOrCustomTopic {
        &self.topic
    }

    fn qos(&'a self) -> i32 {
        self.qos
    }
}

impl<'a> ChannelDefinition<'a> for ChannelReceiverDefinition {
    fn name(&'a self) -> &'a str {
        &self.name
    }

    fn generated_topic(&'a self) -> &'a str {
        &self.generated_topic
    }

    fn topic(&'a self) -> &'a TetherOrCustomTopic {
        &self.topic
    }

    fn qos(&'a self) -> i32 {
        self.qos
    }
}

#[cfg(test)]
mod tests {

    fn verbose_logging() {
        use env_logger::{Builder, Env};
        let mut logger_builder = Builder::from_env(Env::default().default_filter_or("debug"));
        logger_builder.init();
    }

    use crate::{
        ChannelBuilder, ChannelDefinition, ChannelReceiverBuilder, ChannelSenderBuilder,
        TetherAgentBuilder,
    };

    #[test]
    fn default_receiver_channel() {
        // verbose_logging();
        let tether_agent = TetherAgentBuilder::new("tester")
            .auto_connect(false)
            .build()
            .expect("sorry, these tests require working localhost Broker");
        let receiver = ChannelReceiverBuilder::new("one").build(&tether_agent);
        assert_eq!(&receiver.name, "one");
        assert_eq!(&receiver.generated_topic, "+/one/#");
    }

    #[test]
    /// This is a fairly trivial example, but contrast with the test
    /// `sender_channel_default_but_agent_id_custom`: a custom ID was set for the
    /// Agent, so does get added into the Topic for a Channel Receiver created without any
    /// explicit overrides.
    fn default_channel_receiver_with_agent_custom_id() {
        // verbose_logging();
        let tether_agent = TetherAgentBuilder::new("tester")
            .auto_connect(false)
            .id(Some("verySpecialGroup"))
            .build()
            .expect("sorry, these tests require working localhost Broker");
        let receiver = tether_agent.create_receiver::<u8>("one").unwrap();
        assert_eq!(receiver.definition().name(), "one");
        assert_eq!(
            receiver.definition().generated_topic(),
            "+/one/verySpecialGroup"
        );
    }

    #[test]
    fn default_channel_sender() {
        let tether_agent = TetherAgentBuilder::new("tester")
            .auto_connect(false)
            .build()
            .expect("sorry, these tests require working localhost Broker");
        let channel = tether_agent.create_sender::<u8>("two");
        assert_eq!(channel.definition().name(), "two");
        assert_eq!(channel.definition().generated_topic(), "tester/two");
    }

    #[test]
    /// This is identical to the case in which a Channel Sender is created with defaults (no overrides),
    /// BUT the Agent had a custom ID set, which means that the final topic includes this custom
    /// ID/Group value.
    fn sender_channel_default_but_agent_id_custom() {
        let tether_agent = TetherAgentBuilder::new("tester")
            .auto_connect(false)
            .id(Some("specialCustomGrouping"))
            .build()
            .expect("sorry, these tests require working localhost Broker");
        let channel = tether_agent.create_sender::<u8>("somethingStandard");
        assert_eq!(channel.definition().name(), "somethingStandard");
        assert_eq!(
            channel.definition().generated_topic(),
            "tester/somethingStandard/specialCustomGrouping"
        );
    }

    #[test]
    fn receiver_id_andor_role() {
        let tether_agent = TetherAgentBuilder::new("tester")
            .auto_connect(false)
            .build()
            .expect("sorry, these tests require working localhost Broker");

        let receive_role_only = ChannelReceiverBuilder::new("theChannel")
            .role(Some("specificRole"))
            .build(&tether_agent);
        assert_eq!(receive_role_only.name(), "theChannel");
        assert_eq!(
            receive_role_only.generated_topic(),
            "specificRole/theChannel/#"
        );

        let receiver_id_only = ChannelReceiverBuilder::new("theChannel")
            .id(Some("specificID"))
            .build(&tether_agent);
        assert_eq!(receiver_id_only.name(), "theChannel");
        assert_eq!(
            receiver_id_only.generated_topic(),
            "+/theChannel/specificID"
        );

        let receiver_both_custom = ChannelReceiverBuilder::new("theChannel")
            .id(Some("specificID"))
            .role(Some("specificRole"))
            .build(&tether_agent);
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
        let tether_agent = TetherAgentBuilder::new("tester")
            .auto_connect(false)
            .build()
            .expect("sorry, these tests require working localhost Broker");

        let receiver_role_only = ChannelReceiverBuilder::new("theChannel")
            .role(Some("specificRole"))
            .build(&tether_agent);
        assert_eq!(receiver_role_only.name(), "theChannel");
        assert_eq!(
            receiver_role_only.generated_topic(),
            "specificRole/theChannel/#"
        );

        let receiver_id_only = ChannelReceiverBuilder::new("theChannel")
            .id(Some("specificID"))
            .build(&tether_agent);
        assert_eq!(receiver_id_only.name(), "theChannel");
        assert_eq!(
            receiver_id_only.generated_topic(),
            "+/theChannel/specificID"
        );

        let receiver_both = ChannelReceiverBuilder::new("theChannel")
            .id(Some("specificID"))
            .role(Some("specificRole"))
            .build(&tether_agent);
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
    /// position instead".
    fn receiver_specific_id_andor_role_no_channel_name() {
        verbose_logging();
        let tether_agent = TetherAgentBuilder::new("tester")
            .auto_connect(false)
            .build()
            .expect("sorry, these tests require working localhost Broker");

        let receiver_role_only = ChannelReceiverBuilder::new("theOriginalChannel")
            .override_name(Some("+"))
            .role(Some("specificRole"))
            .build(&tether_agent);
        assert_eq!(receiver_role_only.name(), "+");
        assert_eq!(receiver_role_only.generated_topic(), "specificRole/+/#");

        let receiver_id_only = ChannelReceiverBuilder::new("+")
            // .name(Some("+"))
            .any_channel() // equivalent to Some("+")
            .id(Some("specificID"))
            .build(&tether_agent);
        assert_eq!(receiver_id_only.name(), "+");
        assert_eq!(receiver_id_only.generated_topic(), "+/+/specificID");

        let receiver_both = ChannelReceiverBuilder::new("+")
            .id(Some("specificID"))
            .role(Some("specificRole"))
            .build(&tether_agent);
        assert_eq!(receiver_both.name(), "+");
        assert_eq!(receiver_both.generated_topic(), "specificRole/+/specificID");
    }

    #[test]
    /// Some fairly niche cases here
    fn any_name_but_specify_role() {
        let tether_agent = TetherAgentBuilder::new("tester")
            .auto_connect(false)
            .build()
            .expect("sorry, these tests require working localhost Broker");

        let receiver_any_channel = ChannelReceiverBuilder::new("aTest")
            .any_channel()
            .build(&tether_agent);

        assert_eq!(receiver_any_channel.name(), "+");
        assert_eq!(receiver_any_channel.generated_topic(), "+/+/#");

        let receiver_specify_role = ChannelReceiverBuilder::new("aTest")
            .any_channel()
            .role(Some("brain"))
            .build(&tether_agent);

        assert_eq!(receiver_specify_role.name(), "+");
        assert_eq!(receiver_specify_role.generated_topic(), "brain/+/#");
    }

    #[test]
    fn sender_custom() {
        let tether_agent = TetherAgentBuilder::new("tester")
            .auto_connect(false)
            .build()
            .expect("sorry, these tests require working localhost Broker");

        let sender_custom_role = ChannelSenderBuilder::new("theChannelSender")
            .role(Some("customRole"))
            .build(&tether_agent);
        assert_eq!(sender_custom_role.name(), "theChannelSender");
        assert_eq!(
            sender_custom_role.generated_topic(),
            "customRole/theChannelSender"
        );

        let sender_custom_id = ChannelSenderBuilder::new("theChannelSender")
            .id(Some("customID"))
            .build(&tether_agent);
        assert_eq!(sender_custom_id.name(), "theChannelSender");
        assert_eq!(
            sender_custom_id.generated_topic(),
            "tester/theChannelSender/customID"
        );

        let sender_custom_both = ChannelSenderBuilder::new("theChannelSender")
            .role(Some("customRole"))
            .id(Some("customID"))
            .build(&tether_agent);
        assert_eq!(sender_custom_both.name(), "theChannelSender");
        assert_eq!(
            sender_custom_both.generated_topic(),
            "customRole/theChannelSender/customID"
        );
    }

    #[test]
    fn receiver_manual_topics() {
        let tether_agent = TetherAgentBuilder::new("tester")
            .auto_connect(false)
            .build()
            .expect("sorry, these tests require working localhost Broker");

        let receiver_all = ChannelReceiverBuilder::new("everything")
            .override_topic(Some("#"))
            .build(&tether_agent);
        assert_eq!(receiver_all.name(), "everything");
        assert_eq!(receiver_all.generated_topic(), "#");

        let receiver_nontether = ChannelReceiverBuilder::new("weird")
            .override_topic(Some("foo/bar/baz/one/two/three"))
            .build(&tether_agent);
        assert_eq!(receiver_nontether.name(), "weird");
        assert_eq!(
            receiver_nontether.generated_topic(),
            "foo/bar/baz/one/two/three"
        );
    }
}
