use super::tether_compliant_topic::TetherOrCustomTopic;

pub mod receiver_def_builder;
pub mod sender_def_builder;

pub use receiver_def_builder::ChannelReceiverDefBuilder;
use rumqttc::QoS;
pub use sender_def_builder::ChannelSenderDefBuilder;
use serde::{Deserialize, Serialize};

/**
A Channel Def(inition) Builder is used for creating a Channel Def(inition).
*/
pub trait ChannelDefBuilder {
    fn new(name: &str) -> Self;
    fn qos(self, qos: Option<QoS>) -> Self;
    fn role(self, role: Option<&str>) -> Self;
    fn id(self, id: Option<&str>) -> Self;
    fn override_name(self, override_channel_name: Option<&str>) -> Self;
    fn override_topic(self, override_topic: Option<&str>) -> Self;
}

/**
A Channel Def(inition) is intended to encapsulate only the essential metadata
and configuration needed to describe a Channel. In contrast with a Channel Sender/Receiver,
it is **not** responsible for actually sending or receiving messages on that Channel.
*/
pub trait ChannelDef<'a> {
    fn name(&'a self) -> &'a str;
    /// Return the generated topic string actually used by the Channel
    fn generated_topic(&'a self) -> &'a str;
    /// Return the custom or Tether-compliant topic
    fn topic(&'a self) -> &'a TetherOrCustomTopic;
    fn qos(&'a self) -> QoS;
}

pub fn number_to_qos(number: u8) -> QoS {
    match number {
        0 => QoS::AtMostOnce,
        1 => QoS::AtLeastOnce,
        2 => QoS::ExactlyOnce,
        _ => QoS::AtMostOnce,
    }
}

pub fn qos_to_number(qos: QoS) -> u8 {
    match qos {
        QoS::AtMostOnce => 0,
        QoS::AtLeastOnce => 1,
        QoS::ExactlyOnce => 2,
    }
}

#[derive(Serialize, Deserialize)]
#[serde(remote = "QoS")]
#[allow(renamed_and_removed_lints)]
#[allow(clippy::enum_variant_names)]
enum QosDef {
    AtMostOnce,
    AtLeastOnce,
    ExactlyOnce,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ChannelSenderDef {
    pub name: String,
    pub generated_topic: String,
    pub topic: TetherOrCustomTopic,
    #[serde(with = "QosDef")]
    pub qos: QoS,
    pub retain: bool,
}

// impl Serialize for ChannelSenderDef {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: serde::Serializer,
//     {
//         let mut state = serializer.serialize_struct("ChannelSenderDef", 5)?;
//         state.serialize_field("name", &self.name)?;
//         state.serialize_field("generated_topic", &self.generated_topic)?;
//         state.serialize_field("topic", &self.topic)?;
//         state.serialize_field("qos", &qos_to_number(self.qos))?;
//         state.serialize_field("retain", &self.retain)?;
//         state.end()
//     }
// }

// impl<'de> Deserialize<'de> for ChannelSenderDef {
//     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
//     where
//         D: serde::Deserializer<'de>,
//     {
//         let state = deserializer.deserialize_struct("ChannelSenderDef", fields, visitor)
//     }
// }

// impl Serialize for QoS {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: serde::Serializer,
//     {
//         todo!()
//     }
// }

// impl<'de> Deserialize<'de> for QoS {
//     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
//     where
//         D: serde::Deserializer<'de>,
//     {
//         todo!()
//     }
// }

impl ChannelSenderDef {
    pub fn retain(&self) -> bool {
        self.retain
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct ChannelReceiverDef {
    pub name: String,
    pub generated_topic: String,
    pub topic: TetherOrCustomTopic,
    #[serde(with = "QosDef")]
    pub qos: QoS,
}

impl<'a> ChannelDef<'a> for ChannelSenderDef {
    fn name(&'a self) -> &'a str {
        &self.name
    }

    fn generated_topic(&'a self) -> &'a str {
        &self.generated_topic
    }

    fn topic(&'a self) -> &'a TetherOrCustomTopic {
        &self.topic
    }

    fn qos(&'a self) -> QoS {
        self.qos
    }
}

impl<'a> ChannelDef<'a> for ChannelReceiverDef {
    fn name(&'a self) -> &'a str {
        &self.name
    }

    fn generated_topic(&'a self) -> &'a str {
        &self.generated_topic
    }

    fn topic(&'a self) -> &'a TetherOrCustomTopic {
        &self.topic
    }

    fn qos(&'a self) -> QoS {
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
        builder::TetherAgentBuilder, ChannelDef, ChannelDefBuilder, ChannelReceiverDefBuilder,
        ChannelSenderDefBuilder,
    };

    #[test]
    fn default_receiver_channel() {
        // verbose_logging();
        let tether_agent = TetherAgentBuilder::new("tester")
            .auto_connect(false)
            .build()
            .expect("sorry, these tests require working localhost Broker");
        let receiver = ChannelReceiverDefBuilder::new("one").build(tether_agent.config());
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

        let receive_role_only = ChannelReceiverDefBuilder::new("theChannel")
            .role(Some("specificRole"))
            .build(tether_agent.config());
        assert_eq!(receive_role_only.name(), "theChannel");
        assert_eq!(
            receive_role_only.generated_topic(),
            "specificRole/theChannel/#"
        );

        let receiver_id_only = ChannelReceiverDefBuilder::new("theChannel")
            .id(Some("specificID"))
            .build(tether_agent.config());
        assert_eq!(receiver_id_only.name(), "theChannel");
        assert_eq!(
            receiver_id_only.generated_topic(),
            "+/theChannel/specificID"
        );

        let receiver_both_custom = ChannelReceiverDefBuilder::new("theChannel")
            .id(Some("specificID"))
            .role(Some("specificRole"))
            .build(tether_agent.config());
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

        let receiver_role_only = ChannelReceiverDefBuilder::new("theChannel")
            .role(Some("specificRole"))
            .build(tether_agent.config());
        assert_eq!(receiver_role_only.name(), "theChannel");
        assert_eq!(
            receiver_role_only.generated_topic(),
            "specificRole/theChannel/#"
        );

        let receiver_id_only = ChannelReceiverDefBuilder::new("theChannel")
            .id(Some("specificID"))
            .build(tether_agent.config());
        assert_eq!(receiver_id_only.name(), "theChannel");
        assert_eq!(
            receiver_id_only.generated_topic(),
            "+/theChannel/specificID"
        );

        let receiver_both = ChannelReceiverDefBuilder::new("theChannel")
            .id(Some("specificID"))
            .role(Some("specificRole"))
            .build(tether_agent.config());
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

        let receiver_role_only = ChannelReceiverDefBuilder::new("theOriginalChannel")
            .override_name(Some("+"))
            .role(Some("specificRole"))
            .build(tether_agent.config());
        assert_eq!(receiver_role_only.name(), "+");
        assert_eq!(receiver_role_only.generated_topic(), "specificRole/+/#");

        let receiver_id_only = ChannelReceiverDefBuilder::new("+")
            // .name(Some("+"))
            .any_channel() // equivalent to Some("+")
            .id(Some("specificID"))
            .build(tether_agent.config());
        assert_eq!(receiver_id_only.name(), "+");
        assert_eq!(receiver_id_only.generated_topic(), "+/+/specificID");

        let receiver_both = ChannelReceiverDefBuilder::new("+")
            .id(Some("specificID"))
            .role(Some("specificRole"))
            .build(tether_agent.config());
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

        let receiver_any_channel = ChannelReceiverDefBuilder::new("aTest")
            .any_channel()
            .build(tether_agent.config());

        assert_eq!(receiver_any_channel.name(), "+");
        assert_eq!(receiver_any_channel.generated_topic(), "+/+/#");

        let receiver_specify_role = ChannelReceiverDefBuilder::new("aTest")
            .any_channel()
            .role(Some("brain"))
            .build(tether_agent.config());

        assert_eq!(receiver_specify_role.name(), "+");
        assert_eq!(receiver_specify_role.generated_topic(), "brain/+/#");
    }

    #[test]
    fn sender_custom() {
        let tether_agent = TetherAgentBuilder::new("tester")
            .auto_connect(false)
            .build()
            .expect("sorry, these tests require working localhost Broker");

        let sender_custom_role = ChannelSenderDefBuilder::new("theChannelSender")
            .role(Some("customRole"))
            .build(tether_agent.config());
        assert_eq!(sender_custom_role.name(), "theChannelSender");
        assert_eq!(
            sender_custom_role.generated_topic(),
            "customRole/theChannelSender"
        );

        let sender_custom_id = ChannelSenderDefBuilder::new("theChannelSender")
            .id(Some("customID"))
            .build(tether_agent.config());
        assert_eq!(sender_custom_id.name(), "theChannelSender");
        assert_eq!(
            sender_custom_id.generated_topic(),
            "tester/theChannelSender/customID"
        );

        let sender_custom_both = ChannelSenderDefBuilder::new("theChannelSender")
            .role(Some("customRole"))
            .id(Some("customID"))
            .build(tether_agent.config());
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

        let receiver_all = ChannelReceiverDefBuilder::new("everything")
            .override_topic(Some("#"))
            .build(tether_agent.config());
        assert_eq!(receiver_all.name(), "everything");
        assert_eq!(receiver_all.generated_topic(), "#");

        let receiver_nontether = ChannelReceiverDefBuilder::new("weird")
            .override_topic(Some("foo/bar/baz/one/two/three"))
            .build(tether_agent.config());
        assert_eq!(receiver_nontether.name(), "weird");
        assert_eq!(
            receiver_nontether.generated_topic(),
            "foo/bar/baz/one/two/three"
        );
    }
}
