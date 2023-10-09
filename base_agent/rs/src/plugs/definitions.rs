use log::{error, warn};
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
            TetherOrCustomTopic::Custom(s) => s.into(),
            TetherOrCustomTopic::Tether(t) => t.topic().into(),
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

    pub fn matches(&self, topic: &str) -> bool {
        match &self.topic {
            TetherOrCustomTopic::Custom(s) => {
                warn!(
                    "Custom topic \"{}\" on Plug \"{}\" cannot be matched automatically; please filter manually for this",
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
                    let matches_plug_name = my_tpt.plug_name().eq(incoming_three_parts.plug_name());
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

// #[derive(Serialize, Deserialize, Debug)]
// pub enum PlugDefinition {
//     InputPlugDefinition(InputPlugDefinition),
//     OutputPlugDefinition(OutputPlugDefinition),
// }

impl PlugDefinition {
    //     pub fn common(&self) -> &PlugDefinitionCommon {
    //         match self {
    //             PlugDefinition::InputPlugDefinition(plug) => &plug.common,
    //             PlugDefinition::OutputPlugDefinition(plug) => &plug.common,
    //         }
    //     }

    //     pub fn common_mut(&mut self) -> &mut PlugDefinitionCommon {
    //         match self {
    //             PlugDefinition::InputPlugDefinition(plug) => &mut plug.common,
    //             PlugDefinition::OutputPlugDefinition(plug) => &mut plug.common,
    //         }
    //     }

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
