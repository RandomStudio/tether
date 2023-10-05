use crate::*;
use log::debug;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
struct PlugOptionsCommon {
    pub name: String,
    pub topic: Option<String>,
    pub qos: Option<i32>,
}

impl PlugOptionsCommon {
    fn new(name: &str) -> Self {
        PlugOptionsCommon {
            name: name.into(),
            topic: None,
            qos: None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PlugDefinitionCommon {
    pub name: String,
    pub topic: String,
    pub qos: i32,
}

pub struct InputPlugOptions {
    common: PlugOptionsCommon,
}

pub struct OutputPlugOptions {
    common: PlugOptionsCommon,
    retain: Option<bool>,
}

/// This is the definition of an Input or Output Plug
/// You should never use this directly; call build()
/// to get a usable Plug
pub enum PlugOptionsBuilder {
    InputPlugOptions(InputPlugOptions),
    OutputPlugOptions(OutputPlugOptions),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InputPlugDefinition {
    common: PlugDefinitionCommon,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OutputPlugDefinition {
    common: PlugDefinitionCommon,
    retain: bool,
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

impl PlugOptionsBuilder {
    fn common(&mut self) -> &mut PlugOptionsCommon {
        match self {
            PlugOptionsBuilder::InputPlugOptions(plug) => &mut plug.common,
            PlugOptionsBuilder::OutputPlugOptions(plug) => &mut plug.common,
        }
    }

    pub fn create_input(name: &str) -> PlugOptionsBuilder {
        PlugOptionsBuilder::InputPlugOptions(InputPlugOptions {
            common: PlugOptionsCommon::new(name),
        })
    }

    pub fn create_output(name: &str) -> PlugOptionsBuilder {
        PlugOptionsBuilder::OutputPlugOptions(OutputPlugOptions {
            common: PlugOptionsCommon::new(name),
            retain: Some(false),
        })
    }

    pub fn qos(mut self, qos: i32) -> Self {
        self.common().qos = Some(qos);
        self
    }

    pub fn topic(mut self, override_topic: &str) -> Self {
        self.common().topic = Some(override_topic.into());
        self
    }

    pub fn retain(self, should_retain: bool) -> Self {
        match self {
            Self::InputPlugOptions(_) => {
                panic!("Cannot set retain flag on Input Plug / subscription")
            }
            Self::OutputPlugOptions(plug) => {
                PlugOptionsBuilder::OutputPlugOptions(OutputPlugOptions {
                    common: plug.common,
                    retain: Some(should_retain),
                })
            }
        }
    }

    pub fn build(self, tether_agent: &TetherAgent) -> anyhow::Result<PlugDefinition> {
        match self {
            Self::InputPlugOptions(plug) => {
                let final_topic = plug
                    .common
                    .topic
                    .unwrap_or(default_subscribe_topic(&plug.common.name));
                let final_qos = plug.common.qos.unwrap_or(1);
                debug!(
                    "Attempt to subscribe for plug named {} ...",
                    plug.common.name
                );
                match tether_agent.client.subscribe(&final_topic, final_qos) {
                    Ok(res) => {
                        debug!("This topic was fine: --{final_topic}--");
                        debug!("Server respond OK for subscribe: {res:?}");
                        Ok(PlugDefinition::InputPlugDefinition(InputPlugDefinition {
                            common: PlugDefinitionCommon {
                                name: plug.common.name,
                                topic: final_topic,
                                qos: final_qos,
                            },
                        }))
                    }
                    Err(e) => Err(e.into()),
                }
            }
            Self::OutputPlugOptions(plug) => {
                let final_topic = plug.common.topic.unwrap_or(build_topic(
                    &tether_agent.role,
                    &tether_agent.id,
                    &plug.common.name,
                ));
                let final_qos = plug.common.qos.unwrap_or(1);
                // TODO: check valid topic before assuming OK?
                Ok(PlugDefinition::OutputPlugDefinition(OutputPlugDefinition {
                    common: PlugDefinitionCommon {
                        name: plug.common.name,
                        topic: final_topic,
                        qos: final_qos,
                    },
                    retain: plug.retain.unwrap_or(false),
                }))
            }
        }
    }
}
