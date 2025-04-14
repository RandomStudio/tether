pub mod options;
pub mod receiver;
pub mod sender;
pub mod tether_compliant_topic;

pub use options::*;

use super::tether_compliant_topic::TetherOrCustomTopic;

pub trait ChannelCommon<'a> {
    fn name(&'a self) -> &'a str;
    /// Return the generated topic string actually used by the Channel
    fn generated_topic(&'a self) -> &'a str;
    /// Return the custom or Tether-compliant topic
    fn topic(&'a self) -> &'a TetherOrCustomTopic;
    fn qos(&'a self) -> i32;
}

// pub enum TetherChannel {
//     ChannelReceiver(ChannelReceiver),
//     ChannelSender(ChannelSender),
// }

// impl<T> ChannelCommon< for T
// where
//     T: ChannelCommon,
// {
//     pub fn name(&self) -> &str {
//         match self {
//             TetherChannel::ChannelReceiver(p) => p.name(),
//             TetherChannel::ChannelSender(p) => p.name(),
//         }
//     }

//     pub fn generated_topic(&self) -> &str {
//         match self {
//             TetherChannel::ChannelReceiver(p) => p.generated_topic(),
//             TetherChannel::ChannelSender(p) => p.generated_topic(),
//         }
//     }

//     pub fn matches(&self, topic: &TetherOrCustomTopic) -> bool {
//         match self {
//             TetherChannel::ChannelReceiver(p) => p.matches(topic),
//             TetherChannel::ChannelSender(_) => {
//                 error!("We don't check matches for Channel Senders");
//                 false
//             }
//         }
//     }
// }

// #[cfg(test)]
// mod tests {

//     use crate::{
//         tether_compliant_topic::{parse_channel_name, TetherCompliantTopic, TetherOrCustomTopic},
//         ChannelCommon, ChannelReceiver,
//     };

//     #[test]
//     fn receiver_match_tpt() {
//         let channel_def = ChannelReceiver::new(
//             "testChannel",
//             TetherOrCustomTopic::Tether(TetherCompliantTopic::new_for_subscribe(
//                 "testChannel",
//                 None,
//                 None,
//             )),
//             None,
//         );

//         assert_eq!(&channel_def.name, "testChannel");
//         assert_eq!(channel_def.generated_topic(), "+/testChannel/#");
//         assert_eq!(
//             parse_channel_name("someRole/testChannel"),
//             Some("testChannel")
//         );
//         assert_eq!(
//             parse_channel_name("someRole/testChannel/something"),
//             Some("testChannel")
//         );
//         assert!(channel_def.matches(&TetherOrCustomTopic::Tether(
//             TetherCompliantTopic::new_three("dummy", "testChannel", "#")
//         )));
//         assert!(!channel_def.matches(&TetherOrCustomTopic::Tether(
//             TetherCompliantTopic::new_three("dummy", "anotherChannel", "#")
//         )));
//     }

//     #[test]
//     fn receiver_match_tpt_custom_role() {
//         let channel_def = ChannelReceiver::new(
//             "customChanel",
//             TetherOrCustomTopic::Tether(TetherCompliantTopic::new_for_subscribe(
//                 "customChanel",
//                 Some("customRole"),
//                 None,
//             )),
//             None,
//         );

//         assert_eq!(&channel_def.name, "customChanel");
//         assert_eq!(channel_def.generated_topic(), "customRole/customChanel/#");
//         assert!(channel_def.matches(&TetherOrCustomTopic::Tether(
//             TetherCompliantTopic::new_three("customRole", "customChanel", "#")
//         )));
//         assert!(channel_def.matches(&TetherOrCustomTopic::Tether(
//             TetherCompliantTopic::new_three("customRole", "customChanel", "andAnythingElse")
//         )));
//         assert!(!channel_def.matches(&TetherOrCustomTopic::Tether(
//             TetherCompliantTopic::new_three("customRole", "notMyChannel", "#")
//         ))); // wrong incoming Channel Name
//         assert!(!channel_def.matches(&TetherOrCustomTopic::Tether(
//             TetherCompliantTopic::new_three("someOtherRole", "customChanel", "#")
//         ))); // wrong incoming Role
//     }

//     #[test]
//     fn receiver_match_custom_id() {
//         let channel_def = ChannelReceiver::new(
//             "customChanel",
//             TetherOrCustomTopic::Tether(TetherCompliantTopic::new_for_subscribe(
//                 "customChanel",
//                 None,
//                 Some("specificID"),
//             )),
//             None,
//         );

//         assert_eq!(&channel_def.name, "customChanel");
//         assert_eq!(channel_def.generated_topic(), "+/customChanel/specificID");
//         assert!(channel_def.matches(&TetherOrCustomTopic::Tether(
//             TetherCompliantTopic::new_three("anyRole", "customChanel", "specificID",)
//         )));
//         assert!(channel_def.matches(&TetherOrCustomTopic::Tether(
//             TetherCompliantTopic::new_three("anotherRole", "customChanel", "specificID",)
//         ))); // wrong incoming Role
//         assert!(!channel_def.matches(&TetherOrCustomTopic::Tether(
//             TetherCompliantTopic::new_three("anyRole", "notMyChannel", "specificID",)
//         ))); // wrong incoming Channel Name
//         assert!(!channel_def.matches(&TetherOrCustomTopic::Tether(
//             TetherCompliantTopic::new_three("anyRole", "customChanel", "anotherID",)
//         ))); // wrong incoming ID
//     }

//     #[test]
//     fn receiver_match_both() {
//         let channel_def = ChannelReceiver::new(
//             "customChanel",
//             TetherOrCustomTopic::Tether(TetherCompliantTopic::new_for_subscribe(
//                 "customChanel",
//                 Some("specificRole"),
//                 Some("specificID"),
//             )),
//             None,
//         );

//         assert_eq!(&channel_def.name, "customChanel");
//         assert_eq!(
//             channel_def.generated_topic(),
//             "specificRole/customChanel/specificID"
//         );
//         assert!(channel_def.matches(&TetherOrCustomTopic::Tether(
//             TetherCompliantTopic::new_three("specificRole", "customChanel", "specificID",)
//         )));
//         assert!(!channel_def.matches(&TetherOrCustomTopic::Custom(
//             "specificRole/notMyChannel/specificID".into()
//         ))); // wrong incoming Channel Name
//         assert!(!channel_def.matches(&TetherOrCustomTopic::Custom(
//             "specificRole/customChanel/anotherID".into()
//         ))); // wrong incoming ID
//         assert!(!channel_def.matches(&TetherOrCustomTopic::Custom(
//             "anotherRole/customChanel/anotherID".into()
//         ))); // wrong incoming Role
//     }

//     #[test]
//     fn receiver_match_custom_topic() {
//         let channel_def = ChannelReceiver::new(
//             "customChanel",
//             TetherOrCustomTopic::Custom("one/two/three/four/five".into()), // not a standard Tether Three Part Topic
//             None,
//         );

//         assert_eq!(channel_def.name(), "customChanel");
//         // it will match on exactly the same topic:
//         assert!(channel_def.matches(&TetherOrCustomTopic::Custom(
//             "one/two/three/four/five".into()
//         )));

//         // it will NOT match on anything else:
//         assert!(!channel_def.matches(&TetherOrCustomTopic::Custom("one/one/one/one/one".into())));
//     }

//     #[test]
//     fn receiver_match_wildcard() {
//         let channel_def = ChannelReceiver::new(
//             "everything",
//             TetherOrCustomTopic::Custom("#".into()), // fully legal, but not a standard Three Part Topic
//             None,
//         );

//         assert_eq!(channel_def.name(), "everything");

//         // Standard TPT will match
//         assert!(channel_def.matches(&TetherOrCustomTopic::Tether(
//             TetherCompliantTopic::new_three("any", "chanelName", "#")
//         )));

//         // Anything will match, even custom incoming
//         assert!(channel_def.matches(&TetherOrCustomTopic::Custom(
//             "one/two/three/four/five".into()
//         )));
//     }
// }
