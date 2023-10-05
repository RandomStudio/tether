use crate::*;
use anyhow::anyhow;
use log::{debug, error, warn};
use serde::{Deserialize, Serialize};

pub mod definitions;
pub mod options;

pub use definitions::*;
pub use options::*;

#[derive(Serialize, Deserialize, Debug)]
pub struct ThreePartTopic {
    role: String,
    id: String,
    plug_name: String,
}

impl ThreePartTopic {
    /// Publish topics fall back to the ID and/or role associated with the agent, if not explicitly provided
    // pub fn new_for_publish(
    //     role: Option<&str>,
    //     id: Option<&str>,
    //     plug_name: &str,
    //     agent: &TetherAgent,
    // ) -> ThreePartTopic {
    //     ThreePartTopic {
    //         role: String::from(role.unwrap_or(agent.role())),
    //         id: String::from(role.unwrap_or(agent.id())),
    //         plug_name: String::from(plug_name),
    //     }
    // }

    // /// Subscribe topics fall back to wildcard `+` for role and/or id if not explicitly provided
    // pub fn new_for_subscribe(
    //     role: Option<&str>,
    //     id: Option<&str>,
    //     plug_name: &str,
    // ) -> ThreePartTopic {
    //     ThreePartTopic {
    //         role: String::from(role.unwrap_or("+")),
    //         id: String::from(role.unwrap_or("+")),
    //         plug_name: String::from(plug_name),
    //     }
    // }

    pub fn new(role: &str, id: &str, plug_name: &str) -> ThreePartTopic {
        ThreePartTopic {
            role: role.into(),
            id: id.into(),
            plug_name: plug_name.into(),
        }
    }

    pub fn topic(&self) -> String {
        build_topic(&self.role, &self.id, &self.plug_name)
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
            return Err(anyhow!(
                "Did not find exactly three parts in the topic {}",
                value
            ));
        } else {
            debug!("parts: {:?}", parts);
        }

        let role = parts.get(0).expect("this part should exist");
        let id = parts.get(1).expect("this part should exist");
        let plug_name = parts.get(2).expect("this part should exist");

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

fn build_topic(role: &str, id: &str, plug_name: &str) -> String {
    format!("{role}/{id}/{plug_name}")
}

// pub fn default_subscribe_topic(plug_name: &str) -> String {
//     format!("+/+/{plug_name}")
// }

#[derive(Debug)]
enum TetherOrCustomTopic {
    NotSet(),
    TetherTopic(ThreePartTopic),
    CustomTopic(String),
}

// #[derive(Debug)]
// struct PlugOptionsCommon {
//     pub name: String,
//     pub topic: TetherOrCustomTopic,
//     // pub role_override: Option<String>,
//     // pub id_override: Option<String>,
//     pub qos: Option<i32>,
// }

// impl PlugOptionsCommon {
//     fn new(name: &str) -> Self {
//         PlugOptionsCommon {
//             name: name.into(),
//             topic:
//             qos: None,
//         }
//     }
// }
