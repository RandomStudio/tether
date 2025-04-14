pub mod definitions;
pub mod receiver_options;
pub mod sender_options;

pub trait ChannelOptions {
    fn new(name: &str) -> Self;
    fn qos(self, qos: Option<i32>) -> Self;
    fn role(self, role: Option<&str>) -> Self;
    fn id(self, id: Option<&str>) -> Self;
    fn override_name(self, override_channel_name: Option<&str>) -> Self;
    fn override_topic(self, override_topic: Option<&str>) -> Self;
}

// #[cfg(test)]
// mod tests {

//     use crate::{ChannelOptionsBuilder, TetherAgentOptionsBuilder};

//     // fn verbose_logging() {
//     //     use env_logger::{Builder, Env};
//     //     let mut logger_builder = Builder::from_env(Env::default().default_filter_or("debug"));
//     //     logger_builder.init();
//     // }

//     #[test]
//     fn default_receiver_channel() {
//         // verbose_logging();
//         let mut tether_agent = TetherAgentOptionsBuilder::new("tester")
//             .auto_connect(false)
//             .build()
//             .expect("sorry, these tests require working localhost Broker");
//         let receiver = ChannelOptionsBuilder::create_receiver("one")
//             .build(&mut tether_agent)
//             .unwrap();
//         assert_eq!(receiver.name(), "one");
//         assert_eq!(receiver.generated_topic(), "+/one/#");
//     }

//     #[test]
//     /// This is a fairly trivial example, but contrast with the test
//     /// `sender_channel_default_but_agent_id_custom`: although a custom ID was set for the
//     /// Agent, this does not affect the Topic for a Channel Receiver created without any
//     /// explicit overrides.
//     fn default_channel_receiver_with_agent_custom_id() {
//         // verbose_logging();
//         let mut tether_agent = TetherAgentOptionsBuilder::new("tester")
//             .auto_connect(false)
//             .id(Some("verySpecialGroup"))
//             .build()
//             .expect("sorry, these tests require working localhost Broker");
//         let receiver = ChannelOptionsBuilder::create_receiver("one")
//             .build(&mut tether_agent)
//             .unwrap();
//         assert_eq!(receiver.name(), "one");
//         assert_eq!(receiver.generated_topic(), "+/one/#");
//     }

//     #[test]
//     fn default_channel_sender() {
//         let mut tether_agent = TetherAgentOptionsBuilder::new("tester")
//             .auto_connect(false)
//             .build()
//             .expect("sorry, these tests require working localhost Broker");
//         let channel = ChannelOptionsBuilder::create_sender("two")
//             .build(&mut tether_agent)
//             .unwrap();
//         assert_eq!(channel.name(), "two");
//         assert_eq!(channel.generated_topic(), "tester/two");
//     }

//     #[test]
//     /// This is identical to the case in which a Channel Sender is created with defaults (no overrides),
//     /// BUT the Agent had a custom ID set, which means that the final topic includes this custom
//     /// ID/Group value.
//     fn sender_channel_default_but_agent_id_custom() {
//         let mut tether_agent = TetherAgentOptionsBuilder::new("tester")
//             .auto_connect(false)
//             .id(Some("specialCustomGrouping"))
//             .build()
//             .expect("sorry, these tests require working localhost Broker");
//         let channel = ChannelOptionsBuilder::create_sender("somethingStandard")
//             .build(&mut tether_agent)
//             .unwrap();
//         assert_eq!(channel.name(), "somethingStandard");
//         assert_eq!(
//             channel.generated_topic(),
//             "tester/somethingStandard/specialCustomGrouping"
//         );
//     }

//     #[test]
//     fn receiver_id_andor_role() {
//         let mut tether_agent = TetherAgentOptionsBuilder::new("tester")
//             .auto_connect(false)
//             .build()
//             .expect("sorry, these tests require working localhost Broker");

//         let receive_role_only = ChannelOptionsBuilder::create_receiver("theChannel")
//             .role(Some("specificRole"))
//             .build(&mut tether_agent)
//             .unwrap();
//         assert_eq!(receive_role_only.name(), "theChannel");
//         assert_eq!(
//             receive_role_only.generated_topic(),
//             "specificRole/theChannel/#"
//         );

//         let receiver_id_only = ChannelOptionsBuilder::create_receiver("theChannel")
//             .id(Some("specificID"))
//             .build(&mut tether_agent)
//             .unwrap();
//         assert_eq!(receiver_id_only.name(), "theChannel");
//         assert_eq!(
//             receiver_id_only.generated_topic(),
//             "+/theChannel/specificID"
//         );

//         let receiver_both_custom = ChannelOptionsBuilder::create_receiver("theChannel")
//             .id(Some("specificID"))
//             .role(Some("specificRole"))
//             .build(&mut tether_agent)
//             .unwrap();
//         assert_eq!(receiver_both_custom.name(), "theChannel");
//         assert_eq!(
//             receiver_both_custom.generated_topic(),
//             "specificRole/theChannel/specificID"
//         );
//     }

//     #[test]
//     /// If the end-user implicitly specifies the chanel name part (does not set it to Some(_)
//     /// or None) then the ID and/or Role parts will change but the Channel Name part will
//     /// remain the "original" / default
//     /// Contrast with receiver_specific_id_andor_role_no_chanel_name below.
//     fn receiver_specific_id_andor_role_with_channel_name() {
//         let mut tether_agent = TetherAgentOptionsBuilder::new("tester")
//             .auto_connect(false)
//             .build()
//             .expect("sorry, these tests require working localhost Broker");

//         let receiver_role_only = ChannelOptionsBuilder::create_receiver("theChannel")
//             .role(Some("specificRole"))
//             .build(&mut tether_agent)
//             .unwrap();
//         assert_eq!(receiver_role_only.name(), "theChannel");
//         assert_eq!(
//             receiver_role_only.generated_topic(),
//             "specificRole/theChannel/#"
//         );

//         let receiver_id_only = ChannelOptionsBuilder::create_receiver("theChannel")
//             .id(Some("specificID"))
//             .build(&mut tether_agent)
//             .unwrap();
//         assert_eq!(receiver_id_only.name(), "theChannel");
//         assert_eq!(
//             receiver_id_only.generated_topic(),
//             "+/theChannel/specificID"
//         );

//         let receiver_both = ChannelOptionsBuilder::create_receiver("theChannel")
//             .id(Some("specificID"))
//             .role(Some("specificRole"))
//             .build(&mut tether_agent)
//             .unwrap();
//         assert_eq!(receiver_both.name(), "theChannel");
//         assert_eq!(
//             receiver_both.generated_topic(),
//             "specificRole/theChannel/specificID"
//         );
//     }

//     #[test]
//     /// Unlike receiver_specific_id_andor_role_with_channel_name, this tests the situation where
//     /// the end-user (possibly) specifies the ID and/or Role, but also explicitly
//     /// sets the Channel Name to Some("+"), ie. "use a wildcard at this
//     /// position instead" - and NOT the original channel name.
//     fn receiver_specific_id_andor_role_no_channel_name() {
//         let mut tether_agent = TetherAgentOptionsBuilder::new("tester")
//             .auto_connect(false)
//             .build()
//             .expect("sorry, these tests require working localhost Broker");

//         let receiver_only_chanel_name_none = ChannelOptionsBuilder::create_receiver("theChannel")
//             .name(Some("+"))
//             .build(&mut tether_agent)
//             .unwrap();
//         assert_eq!(receiver_only_chanel_name_none.name(), "theChannel");
//         assert_eq!(receiver_only_chanel_name_none.generated_topic(), "+/+/#");

//         let receiver_role_only = ChannelOptionsBuilder::create_receiver("theChannel")
//             .name(Some("+"))
//             .role(Some("specificRole"))
//             .build(&mut tether_agent)
//             .unwrap();
//         assert_eq!(receiver_role_only.name(), "theChannel");
//         assert_eq!(receiver_role_only.generated_topic(), "specificRole/+/#");

//         let receiver_id_only = ChannelOptionsBuilder::create_receiver("theChannel")
//             // .name(Some("+"))
//             .any_channel() // equivalent to Some("+")
//             .id(Some("specificID"))
//             .build(&mut tether_agent)
//             .unwrap();
//         assert_eq!(receiver_id_only.name(), "theChannel");
//         assert_eq!(receiver_id_only.generated_topic(), "+/+/specificID");

//         let receiver_both = ChannelOptionsBuilder::create_receiver("theChannel")
//             .name(Some("+"))
//             .id(Some("specificID"))
//             .role(Some("specificRole"))
//             .build(&mut tether_agent)
//             .unwrap();
//         assert_eq!(receiver_both.name(), "theChannel");
//         assert_eq!(receiver_both.generated_topic(), "specificRole/+/specificID");
//     }

//     #[test]
//     fn any_name_but_specify_role() {
//         // Some fairly niche cases here

//         let mut tether_agent = TetherAgentOptionsBuilder::new("tester")
//             .auto_connect(false)
//             .build()
//             .expect("sorry, these tests require working localhost Broker");

//         let receiver_any_channel = ChannelOptionsBuilder::create_receiver("aTest")
//             .any_channel()
//             .build(&mut tether_agent)
//             .unwrap();

//         assert_eq!(receiver_any_channel.name(), "aTest");
//         assert_eq!(receiver_any_channel.generated_topic(), "+/+/#");

//         let receiver_specify_role = ChannelOptionsBuilder::create_receiver("aTest")
//             .any_channel()
//             .role(Some("brain"))
//             .build(&mut tether_agent)
//             .unwrap();

//         assert_eq!(receiver_specify_role.name(), "aTest");
//         assert_eq!(receiver_specify_role.generated_topic(), "brain/+/#");
//     }

//     #[test]
//     fn sender_custom() {
//         let mut tether_agent = TetherAgentOptionsBuilder::new("tester")
//             .auto_connect(false)
//             .build()
//             .expect("sorry, these tests require working localhost Broker");

//         let sender_custom_role = ChannelOptionsBuilder::create_sender("theChannelSender")
//             .role(Some("customRole"))
//             .build(&mut tether_agent)
//             .unwrap();
//         assert_eq!(sender_custom_role.name(), "theChannelSender");
//         assert_eq!(
//             sender_custom_role.generated_topic(),
//             "customRole/theChannelSender"
//         );

//         let sender_custom_id = ChannelOptionsBuilder::create_sender("theChannelSender")
//             .id(Some("customID"))
//             .build(&mut tether_agent)
//             .unwrap();
//         assert_eq!(sender_custom_id.name(), "theChannelSender");
//         assert_eq!(
//             sender_custom_id.generated_topic(),
//             "tester/theChannelSender/customID"
//         );

//         let sender_custom_both = ChannelOptionsBuilder::create_sender("theChannelSender")
//             .role(Some("customRole"))
//             .id(Some("customID"))
//             .build(&mut tether_agent)
//             .unwrap();
//         assert_eq!(sender_custom_both.name(), "theChannelSender");
//         assert_eq!(
//             sender_custom_both.generated_topic(),
//             "customRole/theChannelSender/customID"
//         );
//     }

//     #[test]
//     fn receiver_manual_topics() {
//         let mut tether_agent = TetherAgentOptionsBuilder::new("tester")
//             .auto_connect(false)
//             .build()
//             .expect("sorry, these tests require working localhost Broker");

//         let receiver_all = ChannelOptionsBuilder::create_receiver("everything")
//             .topic(Some("#"))
//             .build(&mut tether_agent)
//             .unwrap();
//         assert_eq!(receiver_all.name(), "everything");
//         assert_eq!(receiver_all.generated_topic(), "#");

//         let receiver_nontether = ChannelOptionsBuilder::create_receiver("weird")
//             .topic(Some("foo/bar/baz/one/two/three"))
//             .build(&mut tether_agent)
//             .unwrap();
//         assert_eq!(receiver_nontether.name(), "weird");
//         assert_eq!(
//             receiver_nontether.generated_topic(),
//             "foo/bar/baz/one/two/three"
//         );
//     }
// }
