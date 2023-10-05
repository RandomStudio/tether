use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct PlugDefinitionCommon {
    pub name: String,
    pub topic: String,
    pub qos: i32,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct InputPlugDefinition {
    pub common: PlugDefinitionCommon,
}

impl InputPlugDefinition {}

#[derive(Serialize, Deserialize, Debug)]
pub struct OutputPlugDefinition {
    pub common: PlugDefinitionCommon,
    pub retain: bool,
}

impl OutputPlugDefinition {
    pub fn retain(&self) -> bool {
        self.retain
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum PlugDefinition {
    InputPlugDefinition(InputPlugDefinition),
    OutputPlugDefinition(OutputPlugDefinition),
}

impl PlugDefinition {
    pub fn common(&self) -> &PlugDefinitionCommon {
        match self {
            PlugDefinition::InputPlugDefinition(plug) => &plug.common,
            PlugDefinition::OutputPlugDefinition(plug) => &plug.common,
        }
    }

    pub fn common_mut(&mut self) -> &mut PlugDefinitionCommon {
        match self {
            PlugDefinition::InputPlugDefinition(plug) => &mut plug.common,
            PlugDefinition::OutputPlugDefinition(plug) => &mut plug.common,
        }
    }

    pub fn name(&self) -> &str {
        &self.common().name
    }

    pub fn topic(&self) -> &str {
        &self.common().topic
    }
}
