use log::{debug, error};
use serde::{Deserialize, Serialize};

use crate::three_part_topic::ThreePartTopic;

pub trait PlugDefinitionCommon<'a> {
    fn name(&'a self) -> &'a str;
    fn topic(&'a self) -> String;
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

impl<'a> PlugDefinitionCommon<'_> for InputPlugDefinition {
    fn name(&self) -> &str {
        &self.name
    }

    fn topic(&self) -> String {
        match &self.topic {
            TetherOrCustomTopic::Custom(s) => {
                debug!("Plug named \"{}\" has custom topic \"{}\"", &self.name, &s);
                s.into()
            }
            TetherOrCustomTopic::Tether(t) => {
                debug!(
                    "Plug named \"{}\" has Three Part topic \"{:?}\"",
                    &self.name, t
                );
                t.topic().into()
            }
        }
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
    pub fn matches(&self, topic: &str) -> bool {
        match &self.topic {
            TetherOrCustomTopic::Custom(s) => {
                debug!(
                    "Custom/manual topic \"{}\" on Plug \"{}\" cannot be matched automatically; please filter manually for this",
                    &s,
                    self.name()
                );
                true
            }
            TetherOrCustomTopic::Tether(my_tpt) => {
                if let Ok(incoming_three_parts) = ThreePartTopic::try_from(topic) {
                    let matches_role =
                        my_tpt.role() == "+" || my_tpt.role().eq(incoming_three_parts.role());
                    let matches_id =
                        my_tpt.id() == "+" || my_tpt.id().eq(incoming_three_parts.id());
                    let matches_plug_name = my_tpt.plug_name() == "+"
                        || my_tpt.plug_name().eq(incoming_three_parts.plug_name());
                    debug!("Test match for plug named \"{}\" with def {:?} against {:?} => matches_role? {}, matches_id? {}, matches_plug_name? {}", &self.name, &self.topic, &incoming_three_parts, matches_role, matches_id, matches_plug_name);
                    matches_role && matches_id && matches_plug_name
                } else {
                    error!("Incoming topic \"{}\" is not a three-part topic", topic);
                    false
                }
            }
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

    fn topic(&self) -> String {
        match &self.topic {
            TetherOrCustomTopic::Custom(s) => s.into(),
            TetherOrCustomTopic::Tether(t) => t.topic().into(),
        }
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
            PlugDefinition::InputPlug(p) => &p.name(),
            PlugDefinition::OutputPlug(p) => &p.name(),
        }
    }

    pub fn topic(&self) -> String {
        match self {
            PlugDefinition::InputPlug(p) => p.topic(),
            PlugDefinition::OutputPlug(p) => p.topic(),
        }
    }

    pub fn matches(&self, topic: &str) -> bool {
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
        three_part_topic::{parse_agent_id, parse_agent_role, parse_plug_name, ThreePartTopic},
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
        assert_eq!(&plug_def.topic(), "+/+/testPlug");
        assert_eq!(parse_plug_name("+/+/testPlug"), Some("testPlug"));
        assert!(plug_def.matches("dummy/any/testPlug"));
        assert!(!plug_def.matches("dummy/any/anotherPlug"))
    }

    #[test]
    fn input_match_tpt_custom_role() {
        let plug_def = InputPlugDefinition::new(
            "customPlug",
            TetherOrCustomTopic::Tether(ThreePartTopic::new_for_subscribe(
                "customPlug",
                Some("customRole".into()),
                None,
                None,
            )),
            None,
        );

        assert_eq!(&plug_def.name, "customPlug");
        assert_eq!(&plug_def.topic(), "customRole/+/customPlug");
        assert!(plug_def.matches("customRole/any/customPlug"));
        assert!(plug_def.matches("customRole/andAnythingElse/customPlug"));
        assert!(!plug_def.matches("customRole/any/notMyPlug")); // wrong incoming Plug Name
        assert!(!plug_def.matches("someOtherRole/any/customPlug")); // wrong incoming Role
    }

    #[test]
    fn input_match_custom_id() {
        let plug_def = InputPlugDefinition::new(
            "customPlug",
            TetherOrCustomTopic::Tether(ThreePartTopic::new_for_subscribe(
                "customPlug",
                None,
                Some("specificID".into()),
                None,
            )),
            None,
        );

        assert_eq!(&plug_def.name, "customPlug");
        assert_eq!(&plug_def.topic(), "+/specificID/customPlug");
        assert!(plug_def.matches("anyRole/specificID/customPlug"));
        assert!(plug_def.matches("anotherRole/specificID/customPlug"));
        assert!(!plug_def.matches("anyRole/specificID/notMyPlug")); // wrong incoming Plug Name
        assert!(!plug_def.matches("anyRole/anotherID/customPlug")); // wrong incoming ID
    }

    #[test]
    fn input_match_both() {
        let plug_def = InputPlugDefinition::new(
            "customPlug",
            TetherOrCustomTopic::Tether(ThreePartTopic::new_for_subscribe(
                "customPlug",
                Some("specificRole".into()),
                Some("specificID".into()),
                None,
            )),
            None,
        );

        assert_eq!(&plug_def.name, "customPlug");
        assert_eq!(&plug_def.topic(), "specificRole/specificID/customPlug");
        assert!(plug_def.matches("specificRole/specificID/customPlug"));
        assert!(!plug_def.matches("specificRole/specificID/notMyPlug")); // wrong incoming Plug Name
        assert!(!plug_def.matches("specificRole/anotherID/customPlug")); // wrong incoming ID
        assert!(!plug_def.matches("anotherRole/anotherID/customPlug")); // wrong incoming Role
    }

    #[test]
    fn input_match_custom_topic() {
        let plug_def = InputPlugDefinition::new(
            "customPlug",
            TetherOrCustomTopic::Custom("one/two/three".into()),
            None,
        );

        let incoming_topic = String::from("some/other/plug");

        assert_eq!(plug_def.name(), "customPlug");
        assert!(plug_def.matches(&incoming_topic)); // it will match, but emit warning

        assert_eq!(parse_plug_name(&incoming_topic), Some("plug"));
        assert_eq!(parse_agent_role(&incoming_topic), Some("some"));
        assert_eq!(parse_agent_id(&incoming_topic), Some("other"));
    }
}