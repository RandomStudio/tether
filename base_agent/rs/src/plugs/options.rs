use log::{debug, error, info, warn};

use crate::{
    definitions::{
        InputPlugDefinition, OutputPlugDefinition, PlugDefinition, PlugDefinitionCommon,
    },
    TetherAgent, ThreePartTopic,
};

pub struct InputPlugOptions {
    plug_name: String,
    qos: Option<i32>,
    override_subscribe_role: Option<String>,
    override_subscribe_id: Option<String>,
    override_topic: Option<String>,
}

pub struct OutputPlugOptions {
    plug_name: String,
    qos: Option<i32>,
    override_publish_role: Option<String>,
    override_publish_id: Option<String>,
    override_topic: Option<String>,
    retain: Option<bool>,
}

/// This is the definition of an Input or Output Plug
/// You should never use this directly; call build()
/// to get a usable Plug
pub enum PlugOptionsBuilder {
    InputPlugOptions(InputPlugOptions),
    OutputPlugOptions(OutputPlugOptions),
}

impl PlugOptionsBuilder {
    pub fn create_input(name: &str) -> PlugOptionsBuilder {
        PlugOptionsBuilder::InputPlugOptions(InputPlugOptions {
            plug_name: String::from(name),
            override_subscribe_id: None,
            override_subscribe_role: None,
            override_topic: None,
            qos: None,
        })
    }

    pub fn create_output(name: &str) -> PlugOptionsBuilder {
        PlugOptionsBuilder::OutputPlugOptions(OutputPlugOptions {
            plug_name: String::from(name),
            override_publish_id: None,
            override_publish_role: None,
            override_topic: None,
            qos: None,
            retain: None,
        })
    }

    pub fn qos(mut self, qos: i32) -> Self {
        match &mut self {
            PlugOptionsBuilder::InputPlugOptions(s) => s.qos = Some(qos),
            PlugOptionsBuilder::OutputPlugOptions(s) => s.qos = Some(qos),
        };
        self
    }

    pub fn topic(mut self, override_topic: &str) -> Self {
        if TryInto::<ThreePartTopic>::try_into(override_topic).is_ok() {
            info!("Custom topic passes Three Part Topic validation");
        } else {
            warn!(
                "Could not convert \"{}\" into Tether 3 Part Topic; presumably you know what you're doing!",
                override_topic
            );
        }
        match &mut self {
            PlugOptionsBuilder::InputPlugOptions(s) => {
                s.override_topic = Some(override_topic.into())
            }
            PlugOptionsBuilder::OutputPlugOptions(s) => {
                s.override_topic = Some(override_topic.into())
            }
        };
        self
    }

    pub fn retain(mut self, should_retain: bool) -> Self {
        match &mut self {
            Self::InputPlugOptions(_) => {
                error!("Cannot set retain flag on Input Plug / subscription");
            }
            Self::OutputPlugOptions(s) => {
                s.retain = Some(should_retain);
            }
        }
        self
    }

    pub fn build(self, tether_agent: &TetherAgent) -> anyhow::Result<PlugDefinition> {
        match self {
            Self::InputPlugOptions(plug_options) => {
                let final_topic: String = match plug_options.override_topic {
                    Some(s) => s,
                    None => {
                        let t = ThreePartTopic::new(
                            &plug_options
                                .override_subscribe_role
                                .unwrap_or("+".to_string()),
                            &plug_options
                                .override_subscribe_id
                                .unwrap_or("+".to_string()),
                            &plug_options.plug_name,
                        );
                        t.topic()
                    }
                };
                let final_qos = plug_options.qos.unwrap_or(1);
                debug!(
                    "Attempt to subscribe for plug named \"{}\" with topic \"{}\" ...",
                    &plug_options.plug_name, &final_topic
                );
                match tether_agent.client().subscribe(&final_topic, final_qos) {
                    Ok(res) => {
                        debug!("This topic was fine: \"{final_topic}\"",);
                        debug!("Server respond OK for subscribe: {res:?}");
                        Ok(PlugDefinition::InputPlugDefinition(InputPlugDefinition {
                            common: PlugDefinitionCommon {
                                name: plug_options.plug_name,
                                qos: final_qos,
                                topic: final_topic,
                            },
                        }))
                    }
                    Err(e) => Err(e.into()),
                }
            }
            Self::OutputPlugOptions(plug_options) => {
                let final_topic: String = match plug_options.override_topic {
                    Some(s) => s,
                    None => {
                        let t = ThreePartTopic::new(
                            &plug_options
                                .override_publish_role
                                .unwrap_or(tether_agent.id().to_string()),
                            &plug_options
                                .override_publish_id
                                .unwrap_or(tether_agent.id().to_string()),
                            &plug_options.plug_name,
                        );
                        t.topic()
                    }
                };

                let final_qos = plug_options.qos.unwrap_or(1);

                debug!("Creating plug definition (immediately) for plug named \"{}\" with topic \"{}\"", 
              &plug_options.plug_name, &final_topic);

                Ok(PlugDefinition::OutputPlugDefinition(OutputPlugDefinition {
                    common: PlugDefinitionCommon {
                        name: plug_options.plug_name,
                        topic: final_topic,
                        qos: final_qos,
                    },
                    retain: plug_options.retain.unwrap_or(false),
                }))
            }
        }
    }
}
