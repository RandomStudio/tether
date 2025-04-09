use log::{debug, error, warn};
use serde::{Deserialize, Serialize};

use super::tether_compliant_topic::TetherOrCustomTopic;

pub trait ChannelDefinitionCommon<'a> {
    fn name(&'a self) -> &'a str;
    /// Return the generated topic string actually used by the Channel
    fn generated_topic(&'a self) -> &'a str;
    /// Return the custom or Tether-compliant topic
    fn topic(&'a self) -> &'a TetherOrCustomTopic;
    fn qos(&'a self) -> i32;
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ChannelReceiverDefinition {
    name: String,
    topic: TetherOrCustomTopic,
    qos: i32,
}

impl ChannelDefinitionCommon<'_> for ChannelReceiverDefinition {
    fn name(&self) -> &str {
        &self.name
    }

    fn generated_topic(&self) -> &str {
        match &self.topic {
            TetherOrCustomTopic::Custom(s) => {
                debug!(
                    "Channel named \"{}\" has custom topic \"{}\"",
                    &self.name, &s
                );
                s
            }
            TetherOrCustomTopic::Tether(t) => {
                debug!(
                    "Channel named \"{}\" has Tether-compliant topic \"{:?}\"",
                    &self.name, t
                );
                t.topic()
            }
        }
    }

    fn topic(&'_ self) -> &'_ TetherOrCustomTopic {
        &self.topic
    }

    fn qos(&self) -> i32 {
        self.qos
    }
}

impl ChannelReceiverDefinition {
    pub fn new(
        name: &str,
        topic: TetherOrCustomTopic,
        qos: Option<i32>,
    ) -> ChannelReceiverDefinition {
        ChannelReceiverDefinition {
            name: String::from(name),
            topic,
            qos: qos.unwrap_or(1),
        }
    }

    /// Use the topic of an incoming message to check against the definition of an Channel Receiver.
    ///
    /// Due to the use of wildcard subscriptions, multiple topic strings might match a given
    /// Channel Receiver definition. e.g. `someRole/channelMessages` and `anotherRole/channelMessages` and `someRole/channelMessages/specificID`
    /// should ALL match on an Channel Receiver named `channelMessages` unless more specific Role and/or ID
    /// parts were specified in the Channel Receiver Definition.
    ///
    /// In the case where an Channel Receiver was defined with a completely manually-specified topic string,
    /// this function returns a warning and marks ANY incoming message as a valid match; the end-user
    /// developer is expected to match against topic strings themselves.
    pub fn matches(&self, incoming_topic: &TetherOrCustomTopic) -> bool {
        match incoming_topic {
            TetherOrCustomTopic::Tether(incoming_three_parts) => match &self.topic {
                TetherOrCustomTopic::Tether(my_tpt) => {
                    let matches_role =
                        my_tpt.role() == "+" || my_tpt.role().eq(incoming_three_parts.role());
                    let matches_channel_name = my_tpt.channel_name() == "+"
                        || my_tpt
                            .channel_name()
                            .eq(incoming_three_parts.channel_name());
                    let matches_id = match my_tpt.id() {
                        Some(specified_id) => match incoming_three_parts.id() {
                            Some(incoming_id) => specified_id == incoming_id,
                            None => false,
                        },
                        None => true,
                    };

                    debug!("Test match for Channel named \"{}\" with def {:?} against {:?} => matches_role? {}, matches_id? {}, matches_channel_name? {}", &self.name, &self.topic, &incoming_three_parts, matches_role, matches_id, matches_channel_name);
                    matches_role && matches_id && matches_channel_name
                }
                TetherOrCustomTopic::Custom(my_custom_topic) => {
                    debug!(
                    "Custom/manual topic \"{}\" on Channel \"{}\" cannot be matched automatically; please filter manually for this",
                    &my_custom_topic,
                    self.name()
                );
                    my_custom_topic.as_str() == "#"
                        || my_custom_topic.as_str() == incoming_three_parts.topic()
                }
            },
            TetherOrCustomTopic::Custom(incoming_custom) => match &self.topic {
                TetherOrCustomTopic::Custom(my_custom_topic) => {
                    if my_custom_topic.as_str() == "#"
                        || my_custom_topic.as_str() == incoming_custom.as_str()
                    {
                        true
                    } else {
                        warn!(
                            "Incoming topic \"{}\" is not a Tether-Compliant topic",
                            &incoming_custom
                        );
                        false
                    }
                }
                TetherOrCustomTopic::Tether(_) => {
                    error!("Incoming is NOT Tether Compliant Topic but this Channel DOES have Tether Compliant Topic; cannot decide match");
                    false
                }
            },
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ChannelSenderDefinition {
    name: String,
    topic: TetherOrCustomTopic,
    qos: i32,
    retain: bool,
}

impl ChannelDefinitionCommon<'_> for ChannelSenderDefinition {
    fn name(&'_ self) -> &'_ str {
        &self.name
    }

    fn generated_topic(&self) -> &str {
        match &self.topic {
            TetherOrCustomTopic::Custom(s) => s,
            TetherOrCustomTopic::Tether(t) => t.topic(),
        }
    }

    fn topic(&'_ self) -> &'_ TetherOrCustomTopic {
        &self.topic
    }

    fn qos(&'_ self) -> i32 {
        self.qos
    }
}

impl ChannelSenderDefinition {
    pub fn new(
        name: &str,
        topic: TetherOrCustomTopic,
        qos: Option<i32>,
        retain: Option<bool>,
    ) -> ChannelSenderDefinition {
        ChannelSenderDefinition {
            name: String::from(name),
            topic,
            qos: qos.unwrap_or(1),
            retain: retain.unwrap_or(false),
        }
    }

    pub fn retain(&self) -> bool {
        self.retain
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ChannelDefinition {
    ChannelReceiver(ChannelReceiverDefinition),
    ChannelSender(ChannelSenderDefinition),
}

impl ChannelDefinition {
    pub fn name(&self) -> &str {
        match self {
            ChannelDefinition::ChannelReceiver(p) => p.name(),
            ChannelDefinition::ChannelSender(p) => p.name(),
        }
    }

    pub fn generated_topic(&self) -> &str {
        match self {
            ChannelDefinition::ChannelReceiver(p) => p.generated_topic(),
            ChannelDefinition::ChannelSender(p) => p.generated_topic(),
        }
    }

    pub fn matches(&self, topic: &TetherOrCustomTopic) -> bool {
        match self {
            ChannelDefinition::ChannelReceiver(p) => p.matches(topic),
            ChannelDefinition::ChannelSender(_) => {
                error!("We don't check matches for Channel Senders");
                false
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::{
        tether_compliant_topic::{parse_channel_name, TetherCompliantTopic, TetherOrCustomTopic},
        ChannelDefinitionCommon, ChannelReceiverDefinition,
    };

    #[test]
    fn receiver_match_tpt() {
        let channel_def = ChannelReceiverDefinition::new(
            "testChannel",
            TetherOrCustomTopic::Tether(TetherCompliantTopic::new_for_subscribe(
                "testChannel",
                None,
                None,
            )),
            None,
        );

        assert_eq!(&channel_def.name, "testChannel");
        assert_eq!(channel_def.generated_topic(), "+/testChannel/#");
        assert_eq!(
            parse_channel_name("someRole/testChannel"),
            Some("testChannel")
        );
        assert_eq!(
            parse_channel_name("someRole/testChannel/something"),
            Some("testChannel")
        );
        assert!(channel_def.matches(&TetherOrCustomTopic::Tether(
            TetherCompliantTopic::new_three("dummy", "testChannel", "#")
        )));
        assert!(!channel_def.matches(&TetherOrCustomTopic::Tether(
            TetherCompliantTopic::new_three("dummy", "anotherChannel", "#")
        )));
    }

    #[test]
    fn receiver_match_tpt_custom_role() {
        let channel_def = ChannelReceiverDefinition::new(
            "customChanel",
            TetherOrCustomTopic::Tether(TetherCompliantTopic::new_for_subscribe(
                "customChanel",
                Some("customRole"),
                None,
            )),
            None,
        );

        assert_eq!(&channel_def.name, "customChanel");
        assert_eq!(channel_def.generated_topic(), "customRole/customChanel/#");
        assert!(channel_def.matches(&TetherOrCustomTopic::Tether(
            TetherCompliantTopic::new_three("customRole", "customChanel", "#")
        )));
        assert!(channel_def.matches(&TetherOrCustomTopic::Tether(
            TetherCompliantTopic::new_three("customRole", "customChanel", "andAnythingElse")
        )));
        assert!(!channel_def.matches(&TetherOrCustomTopic::Tether(
            TetherCompliantTopic::new_three("customRole", "notMyChannel", "#")
        ))); // wrong incoming Channel Name
        assert!(!channel_def.matches(&TetherOrCustomTopic::Tether(
            TetherCompliantTopic::new_three("someOtherRole", "customChanel", "#")
        ))); // wrong incoming Role
    }

    #[test]
    fn receiver_match_custom_id() {
        let channel_def = ChannelReceiverDefinition::new(
            "customChanel",
            TetherOrCustomTopic::Tether(TetherCompliantTopic::new_for_subscribe(
                "customChanel",
                None,
                Some("specificID"),
            )),
            None,
        );

        assert_eq!(&channel_def.name, "customChanel");
        assert_eq!(channel_def.generated_topic(), "+/customChanel/specificID");
        assert!(channel_def.matches(&TetherOrCustomTopic::Tether(
            TetherCompliantTopic::new_three("anyRole", "customChanel", "specificID",)
        )));
        assert!(channel_def.matches(&TetherOrCustomTopic::Tether(
            TetherCompliantTopic::new_three("anotherRole", "customChanel", "specificID",)
        ))); // wrong incoming Role
        assert!(!channel_def.matches(&TetherOrCustomTopic::Tether(
            TetherCompliantTopic::new_three("anyRole", "notMyChannel", "specificID",)
        ))); // wrong incoming Channel Name
        assert!(!channel_def.matches(&TetherOrCustomTopic::Tether(
            TetherCompliantTopic::new_three("anyRole", "customChanel", "anotherID",)
        ))); // wrong incoming ID
    }

    #[test]
    fn receiver_match_both() {
        let channel_def = ChannelReceiverDefinition::new(
            "customChanel",
            TetherOrCustomTopic::Tether(TetherCompliantTopic::new_for_subscribe(
                "customChanel",
                Some("specificRole"),
                Some("specificID"),
            )),
            None,
        );

        assert_eq!(&channel_def.name, "customChanel");
        assert_eq!(
            channel_def.generated_topic(),
            "specificRole/customChanel/specificID"
        );
        assert!(channel_def.matches(&TetherOrCustomTopic::Tether(
            TetherCompliantTopic::new_three("specificRole", "customChanel", "specificID",)
        )));
        assert!(!channel_def.matches(&TetherOrCustomTopic::Custom(
            "specificRole/notMyChannel/specificID".into()
        ))); // wrong incoming Channel Name
        assert!(!channel_def.matches(&TetherOrCustomTopic::Custom(
            "specificRole/customChanel/anotherID".into()
        ))); // wrong incoming ID
        assert!(!channel_def.matches(&TetherOrCustomTopic::Custom(
            "anotherRole/customChanel/anotherID".into()
        ))); // wrong incoming Role
    }

    #[test]
    fn receiver_match_custom_topic() {
        let channel_def = ChannelReceiverDefinition::new(
            "customChanel",
            TetherOrCustomTopic::Custom("one/two/three/four/five".into()), // not a standard Tether Three Part Topic
            None,
        );

        assert_eq!(channel_def.name(), "customChanel");
        // it will match on exactly the same topic:
        assert!(channel_def.matches(&TetherOrCustomTopic::Custom(
            "one/two/three/four/five".into()
        )));

        // it will NOT match on anything else:
        assert!(!channel_def.matches(&TetherOrCustomTopic::Custom("one/one/one/one/one".into())));
    }

    #[test]
    fn receiver_match_wildcard() {
        let channel_def = ChannelReceiverDefinition::new(
            "everything",
            TetherOrCustomTopic::Custom("#".into()), // fully legal, but not a standard Three Part Topic
            None,
        );

        assert_eq!(channel_def.name(), "everything");

        // Standard TPT will match
        assert!(channel_def.matches(&TetherOrCustomTopic::Tether(
            TetherCompliantTopic::new_three("any", "chanelName", "#")
        )));

        // Anything will match, even custom incoming
        assert!(channel_def.matches(&TetherOrCustomTopic::Custom(
            "one/two/three/four/five".into()
        )));
    }
}
