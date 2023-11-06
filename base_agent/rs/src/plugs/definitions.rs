use log::{debug, error, warn};
use serde::{Deserialize, Serialize};

use crate::three_part_topic::ThreePartTopic;

pub trait PlugDefinitionCommon<'a> {
    fn name(&'a self) -> &'a str;
    fn topic_str(&'a self) -> &'a str;
    fn topic(&'a self) -> &'a TetherOrCustomTopic;
    fn qos(&'a self) -> i32;
}

#[derive(Serialize, Deserialize, Debug)]
pub enum TetherOrCustomTopic {
    Tether(ThreePartTopic),
    Custom(String),
}
#[derive(Serialize, Deserialize, Debug)]
pub struct InputPlugDefinition {
    name: String,
    topic: TetherOrCustomTopic,
    qos: i32,
}

impl PlugDefinitionCommon<'_> for InputPlugDefinition {
    fn name(&self) -> &str {
        &self.name
    }

    fn topic_str(&self) -> &str {
        match &self.topic {
            TetherOrCustomTopic::Custom(s) => {
                debug!("Plug named \"{}\" has custom topic \"{}\"", &self.name, &s);
                s
            }
            TetherOrCustomTopic::Tether(t) => {
                debug!(
                    "Plug named \"{}\" has Three Part topic \"{:?}\"",
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

impl InputPlugDefinition {
    pub fn new(name: &str, topic: TetherOrCustomTopic, qos: Option<i32>) -> InputPlugDefinition {
        InputPlugDefinition {
            name: String::from(name),
            topic,
            qos: qos.unwrap_or(1),
        }
    }

    /// Use the topic of an incoming message to check against the definition of an Input Plug.
    ///
    /// Due to the use of wildcard subscriptions, multiple topic strings might match a given
    /// Input Plug definition. e.g. `someRole/any/plugMessages` and `anotherRole/any/plugMessages`
    /// should both match on an Input Plug named `plugMessages` unless more specific Role and/or ID
    /// parts were specified in the Input Plug Definition.
    ///
    /// In the case where an Input Plug was defined with a completely manually-specified topic string,
    /// this function returns a warning and marks ANY incoming message as a valid match; the end-user
    /// developer is expected to match against topic strings themselves.
    pub fn matches(&self, incoming_topic: &TetherOrCustomTopic) -> bool {
        // match &self.topic {
        //     TetherOrCustomTopic::Custom(s) => {
        //         debug!(
        //             "Custom/manual topic \"{}\" on Plug \"{}\" cannot be matched automatically; please filter manually for this",
        //             &s,
        //             self.name()
        //         );
        //         true
        //     }
        // TetherOrCustomTopic::Tether(my_tpt) => {
        match incoming_topic {
            TetherOrCustomTopic::Tether(incoming_three_parts) => match &self.topic {
                TetherOrCustomTopic::Tether(my_tpt) => {
                    let matches_role =
                        my_tpt.role() == "+" || my_tpt.role().eq(incoming_three_parts.role());
                    let matches_id =
                        my_tpt.id() == "+" || my_tpt.id().eq(incoming_three_parts.id());
                    let matches_plug_name = my_tpt.plug_name() == "+"
                        || my_tpt.plug_name().eq(incoming_three_parts.plug_name());
                    debug!("Test match for plug named \"{}\" with def {:?} against {:?} => matches_role? {}, matches_id? {}, matches_plug_name? {}", &self.name, &self.topic, &incoming_three_parts, matches_role, matches_id, matches_plug_name);
                    matches_role && matches_id && matches_plug_name
                }
                TetherOrCustomTopic::Custom(my_custom_topic) => {
                    debug!(
                    "Custom/manual topic \"{}\" on Plug \"{}\" cannot be matched automatically; please filter manually for this",
                    &my_custom_topic,
                    self.name()
                );
                    if my_custom_topic.as_str() == "#"
                        || my_custom_topic.as_str() == incoming_three_parts.topic()
                    {
                        true
                    } else {
                        false
                    }
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
                            "Incoming topic \"{}\" is not a three-part topic",
                            &incoming_custom
                        );
                        false
                    }
                }
                TetherOrCustomTopic::Tether(_) => {
                    error!("Incoming is NOT Three Part Topic but this plug DOES have Three Part Topic; cannot decide match");
                    false
                }
            },
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OutputPlugDefinition {
    name: String,
    topic: TetherOrCustomTopic,
    qos: i32,
    retain: bool,
}

impl PlugDefinitionCommon<'_> for OutputPlugDefinition {
    fn name(&'_ self) -> &'_ str {
        &self.name
    }

    fn topic_str(&self) -> &str {
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

impl OutputPlugDefinition {
    pub fn new(
        name: &str,
        topic: TetherOrCustomTopic,
        qos: Option<i32>,
        retain: Option<bool>,
    ) -> OutputPlugDefinition {
        OutputPlugDefinition {
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
pub enum PlugDefinition {
    InputPlug(InputPlugDefinition),
    OutputPlug(OutputPlugDefinition),
}

impl PlugDefinition {
    pub fn name(&self) -> &str {
        match self {
            PlugDefinition::InputPlug(p) => p.name(),
            PlugDefinition::OutputPlug(p) => p.name(),
        }
    }

    pub fn topic(&self) -> &str {
        match self {
            PlugDefinition::InputPlug(p) => p.topic_str(),
            PlugDefinition::OutputPlug(p) => p.topic_str(),
        }
    }

    pub fn matches(&self, topic: &TetherOrCustomTopic) -> bool {
        match self {
            PlugDefinition::InputPlug(p) => p.matches(topic),
            PlugDefinition::OutputPlug(_) => {
                error!("We don't check matches for Output Plugs");
                false
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::{
        three_part_topic::{parse_plug_name, ThreePartTopic},
        InputPlugDefinition, PlugDefinitionCommon, TetherOrCustomTopic,
    };

    #[test]
    fn input_match_tpt() {
        let plug_def = InputPlugDefinition::new(
            "testPlug",
            TetherOrCustomTopic::Tether(ThreePartTopic::new_for_subscribe(
                "testPlug", None, None, None,
            )),
            None,
        );

        assert_eq!(&plug_def.name, "testPlug");
        assert_eq!(plug_def.topic_str(), "+/+/testPlug");
        assert_eq!(parse_plug_name("+/+/testPlug"), Some("testPlug"));
        assert!(
            plug_def.matches(&TetherOrCustomTopic::Tether(ThreePartTopic::new(
                "dummy", "any", "testPlug"
            )))
        );
        assert!(
            !plug_def.matches(&TetherOrCustomTopic::Tether(ThreePartTopic::new(
                "dummy",
                "any",
                "anotherPlug"
            )))
        );
        // assert!(!plug_def.matches(&TetherOrCustomTopic::Custom("dummy/any/anotherPlug".into())));
    }

    #[test]
    fn input_match_tpt_custom_role() {
        let plug_def = InputPlugDefinition::new(
            "customPlug",
            TetherOrCustomTopic::Tether(ThreePartTopic::new_for_subscribe(
                "customPlug",
                Some("customRole"),
                None,
                None,
            )),
            None,
        );

        assert_eq!(&plug_def.name, "customPlug");
        assert_eq!(plug_def.topic_str(), "customRole/+/customPlug");
        assert!(
            plug_def.matches(&TetherOrCustomTopic::Tether(ThreePartTopic::new(
                "customRole",
                "any",
                "customPlug"
            )))
        );
        assert!(
            plug_def.matches(&TetherOrCustomTopic::Tether(ThreePartTopic::new(
                "customRole",
                "andAnythingElse",
                "customPlug"
            )))
        );
        assert!(
            !plug_def.matches(&TetherOrCustomTopic::Tether(ThreePartTopic::new(
                "customRole",
                "any",
                "notMyPlug"
            )))
        ); // wrong incoming Plug N.into())ame
        assert!(
            !plug_def.matches(&TetherOrCustomTopic::Tether(ThreePartTopic::new(
                "someOtherRole",
                "any",
                "customPlug"
            )))
        ); // wrong incoming R.into())ole
    }

    #[test]
    fn input_match_custom_id() {
        let plug_def = InputPlugDefinition::new(
            "customPlug",
            TetherOrCustomTopic::Tether(ThreePartTopic::new_for_subscribe(
                "customPlug",
                None,
                Some("specificID"),
                None,
            )),
            None,
        );

        assert_eq!(&plug_def.name, "customPlug");
        assert_eq!(plug_def.topic_str(), "+/specificID/customPlug");
        assert!(
            plug_def.matches(&TetherOrCustomTopic::Tether(ThreePartTopic::new(
                "anyRole",
                "specificID",
                "customPlug"
            )))
        );
        assert!(
            plug_def.matches(&TetherOrCustomTopic::Tether(ThreePartTopic::new(
                "anotherRole",
                "specificID",
                "customPlug"
            )))
        ); // wrong incoming Role
        assert!(
            !plug_def.matches(&TetherOrCustomTopic::Tether(ThreePartTopic::new(
                "anyRole",
                "specificID",
                "notMyPlug"
            )))
        ); // wrong incoming Plug Name
        assert!(
            !plug_def.matches(&TetherOrCustomTopic::Tether(ThreePartTopic::new(
                "anyRole",
                "anotherID",
                "customPlug"
            )))
        ); // wrong incoming ID
    }

    #[test]
    fn input_match_both() {
        let plug_def = InputPlugDefinition::new(
            "customPlug",
            TetherOrCustomTopic::Tether(ThreePartTopic::new_for_subscribe(
                "customPlug",
                Some("specificRole"),
                Some("specificID"),
                None,
            )),
            None,
        );

        assert_eq!(&plug_def.name, "customPlug");
        assert_eq!(plug_def.topic_str(), "specificRole/specificID/customPlug");
        assert!(
            plug_def.matches(&TetherOrCustomTopic::Tether(ThreePartTopic::new(
                "specificRole",
                "specificID",
                "customPlug"
            )))
        );
        assert!(!plug_def.matches(&TetherOrCustomTopic::Custom(
            "specificRole/specificID/notMyPlug".into()
        ))); // wrong incoming Plug N.into())ame
        assert!(!plug_def.matches(&TetherOrCustomTopic::Custom(
            "specificRole/anotherID/customPlug".into()
        ))); // wrong incoming.into()) ID
        assert!(!plug_def.matches(&TetherOrCustomTopic::Custom(
            "anotherRole/anotherID/customPlug".into()
        ))); // wrong incoming R.into())ole
    }

    #[test]
    fn input_match_custom_topic() {
        let plug_def = InputPlugDefinition::new(
            "customPlug",
            TetherOrCustomTopic::Custom("one/two/three/four/five".into()), // not a standard Tether Three Part Topic
            None,
        );

        assert_eq!(plug_def.name(), "customPlug");
        // it will match on exactly the same topic:
        assert!(plug_def.matches(&TetherOrCustomTopic::Custom(
            "one/two/three/four/five".into()
        )));

        // it will NOT match on anything else:
        assert!(!plug_def.matches(&TetherOrCustomTopic::Custom("one/one/one/one/one".into())));
    }

    #[test]
    fn input_match_wildcard() {
        let plug_def = InputPlugDefinition::new(
            "everything",
            TetherOrCustomTopic::Custom("#".into()), // fully legal, but not a standard Three Part Topic
            None,
        );

        assert_eq!(plug_def.name(), "everything");

        // Standard TPT will match
        assert!(
            plug_def.matches(&TetherOrCustomTopic::Tether(ThreePartTopic::new(
                "any", "any", "plugName"
            )))
        );

        // Anything will match, even custom incoming
        assert!(plug_def.matches(&TetherOrCustomTopic::Custom(
            "one/two/three/four/five".into()
        )));
    }
}
