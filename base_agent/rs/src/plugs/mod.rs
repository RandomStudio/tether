use crate::*;
use log::{debug, error};
use serde::{Deserialize, Serialize};
use anyhow::anyhow;

#[derive(Serialize, Deserialize, Debug)]
pub struct ThreePartTopic {
    role: String,
    id: String,
    plug_name: String,
}

impl ThreePartTopic {

    /// Publish topics fall back to the ID and/or role associated with the agent, if not explicitly provided
    pub fn new_for_publish (role: Option<String>, id: Option<String>, plug_name: &str, agent: &TetherAgent) -> ThreePartTopic {
        ThreePartTopic { role: role.unwrap_or(agent.role().into()), id: id.unwrap_or(agent.id().into()), plug_name: plug_name.into() }
    }   

    /// Subscribe topics fall back to wildcard `+` for role and/or id if not explicitly provided
    pub fn new_for_subscribe (role: Option<String>, id: Option<String>, plug_name: String) -> ThreePartTopic {
        ThreePartTopic { role: role.unwrap_or("+".into()), id: id.unwrap_or("+".into()), plug_name: plug_name.into() }
    }

    fn new (role: &str, id: &str, plug_name: &str) -> ThreePartTopic {
        ThreePartTopic { role: role.into(), id: id.into(), plug_name: plug_name.into() }
    }

    /// Turn the ThreePartTopic back into an actual topic string. This is rebuilt (and the String is reallocated)
    /// each time the function is called, using the role + ID + plug_name parts. In other words, at no point do we 
    /// retain complete topic string internally.
    pub fn topic(&self) -> String {
        format!("{}/{}/{}", &self.role, &self.id, &self.plug_name)
    }

    pub fn role(&self) -> &str {
        &self.role
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn plug_name(&self) -> &str {
        &self.plug_name
    }

    pub fn set_role(&mut self, role: &str) {
        self.role = role.into();
    }

    pub fn set_id(&mut self, id: &str) {
        self.id = id.into();
    }

    pub fn set_plug_name(&mut self, plug_name: &str) {
        self.plug_name = plug_name.into();
    }


}

impl TryFrom<&str> for ThreePartTopic {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {

        let parts = value.split('/').collect::<Vec<&str>>();

        if parts.len() != 3 {
            return Err(anyhow!("Did not find exactly three parts in the topic {}", value));
        }

        let role = parts.get(0).expect("this part should exist");
        let id = parts.get(1).expect("this part should exist");
        let plug_name = parts.get(3).expect("this part should exist");

        return Ok(ThreePartTopic::new(role, id, plug_name));

    }
}

// pub fn parse_plug_name(topic: &str) -> Option<&str> {
//     let parts: Vec<&str> = topic.split('/').collect();
//     match parts.get(2) {
//         Some(s) => Some(*s),
//         None => None,
//     }
// }

// pub fn parse_agent_id(topic: &str) -> Option<&str> {
//     let parts: Vec<&str> = topic.split('/').collect();
//     match parts.get(1) {
//         Some(s) => Some(*s),
//         None => None,
//     }
// }

// pub fn parse_agent_role(topic: &str) -> Option<&str> {
//     let parts: Vec<&str> = topic.split('/').collect();
//     match parts.first() {
//         Some(s) => Some(*s),
//         None => None,
//     }
// }

// pub fn build_topic(role: &str, id: &str, plug_name: &str) -> String {
//     format!("{role}/{id}/{plug_name}")
// }

// pub fn default_subscribe_topic(plug_name: &str) -> String {
//     format!("+/+/{plug_name}")
// }

#[derive(Debug)]
struct PlugOptionsCommon {
    pub name: String,
    pub role_override: Option<String>,
    pub id_override: Option<String>,
    pub qos: Option<i32>,
}

impl PlugOptionsCommon {
    fn new(name: &str) -> Self {
        PlugOptionsCommon {
            name: name.into(),
            role_override: None,
            id_override: None,
            qos: None,
        }
    }
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
pub struct PlugDefinitionCommon {
    pub name: String,
    pub three_part_topic: ThreePartTopic,
    pub qos: i32,
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

    pub fn topic(&self) -> &ThreePartTopic {
        &self.common().three_part_topic
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

    pub fn set_role(mut self, role: &str) -> Self {
        self.common().role_override = Some(role.into());
        self
    }

    pub fn set_id(mut self, id: &str) -> Self {
        self.common().id_override = Some(id.into());
        self
    }

    pub fn topic(mut self, override_topic: &str) -> Self {
        if let Ok(three_part_topic) = TryInto::<ThreePartTopic>::try_into(override_topic) {
            self.common().role_override = Some(three_part_topic.role().into());
            self.common().id_override = Some(three_part_topic.id().into());
        } else {
            error!("Invalid topic; could not convert into Tether 3 Part Topic");
        }
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
                let three_part_topic = ThreePartTopic::new_for_subscribe(
                    plug.common.role_override.and_then(|s| Some(s)), 
                    plug.common.id_override.and_then(|s| Some(s)), 
                    plug.common.name.clone(),
                );
                let final_qos = plug.common.qos.unwrap_or(1);
                debug!(
                    "Attempt to subscribe for plug named {} ...",
                    &three_part_topic.topic()
                );
                let topic_string = three_part_topic.topic();
                match tether_agent.client().subscribe(&topic_string, final_qos) {
                    Ok(res) => {
                        debug!("This topic was fine: \"{topic_string}\"", );
                        debug!("Server respond OK for subscribe: {res:?}");
                        Ok(PlugDefinition::InputPlugDefinition(InputPlugDefinition {
                            common: PlugDefinitionCommon {
                                name: plug.common.name,
                                three_part_topic,
                                qos: final_qos,
                            },
                        }))
                    }
                    Err(e) => Err(e.into()),
                }
            }
            Self::OutputPlugOptions(plug) => {
                let three_part_topic = ThreePartTopic::new_for_publish(
                    plug.common.role_override.and_then(|s| Some(s)),
                    plug.common.id_override.and_then(|s| Some(s)), 
                    &plug.common.name, tether_agent);

                let final_qos = plug.common.qos.unwrap_or(1);

                Ok(PlugDefinition::OutputPlugDefinition(OutputPlugDefinition {
                    common: PlugDefinitionCommon {
                        name: plug.common.name,
                        three_part_topic,
                        qos: final_qos,
                    },
                    retain: plug.retain.unwrap_or(false),
                }))
            }
        }
    }
}
