use anyhow::anyhow;
use log::*;
use serde::{Deserialize, Serialize};

use crate::TetherAgent;

#[derive(Serialize, Deserialize, Debug)]
pub struct ThreePartTopic {
    role: String,
    id: Option<String>,
    plug_name: String,
    full_topic: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum TetherOrCustomTopic {
    Tether(ThreePartTopic),
    Custom(String),
}

impl TetherOrCustomTopic {
    pub fn full_topic_string(&self) -> String {
        match self {
            TetherOrCustomTopic::Tether(three_part_topic) => String::from(three_part_topic.topic()),
            TetherOrCustomTopic::Custom(t) => String::from(t),
        }
    }
}

impl ThreePartTopic {
    /// Publish topics fall back to the ID and/or role associated with the agent, if not explicitly provided
    pub fn new_for_publish(
        agent: &TetherAgent,
        plug_name: &str,
        role_part_override: Option<&str>,
        id_part_override: Option<&str>,
    ) -> ThreePartTopic {
        let role = role_part_override.unwrap_or(agent.role());
        let full_topic = build_publish_topic(role, plug_name, id_part_override);
        ThreePartTopic {
            role: role.into(),
            id: id_part_override.map(String::from),
            plug_name: plug_name.into(),
            full_topic,
        }
    }

    /// Subscribe topics fall back to wildcard `+` for role if not explicitly provided.
    pub fn new_for_subscribe(
        plug_name: &str,
        role_part_override: Option<&str>,
        id_part_override: Option<&str>,
    ) -> ThreePartTopic {
        let role = role_part_override.unwrap_or("+");
        let full_topic = build_subscribe_topic(role, plug_name, id_part_override);

        ThreePartTopic {
            role: role.into(),
            id: id_part_override.map(String::from),
            plug_name: plug_name.into(),
            full_topic,
        }
    }

    /// Directly constructs a Three Part Topic with explicitly provided role, plug_name, and id.
    pub fn new_three(role: &str, plug_name: &str, id: &str) -> ThreePartTopic {
        ThreePartTopic {
            role: role.into(),
            id: Some(id.into()),
            plug_name: plug_name.into(),
            full_topic: build_subscribe_topic(role, plug_name, Some(id)),
        }
    }

    /// Directly constructs a Three Part Topic with explicitly provided role, plug_name, and id.
    pub fn new_two(role: &str, plug_name: &str) -> ThreePartTopic {
        ThreePartTopic {
            role: role.into(),
            id: None,
            plug_name: plug_name.into(),
            full_topic: format!("{role}/{plug_name}"),
        }
    }

    pub fn topic(&self) -> &str {
        &self.full_topic
    }

    pub fn role(&self) -> &str {
        &self.role
    }

    pub fn id(&self) -> Option<&str> {
        self.id.as_deref()
    }

    pub fn plug_name(&self) -> &str {
        &self.plug_name
    }

    pub fn set_role(&mut self, role: &str) {
        self.role = role.into();
        self.update_full_topic();
    }

    pub fn set_id(&mut self, id: Option<&str>) {
        self.id = id.map(|id| id.into());
        self.update_full_topic();
    }

    pub fn set_plug_name(&mut self, plug_name: &str) {
        self.plug_name = plug_name.into();
        self.update_full_topic();
    }

    fn update_full_topic(&mut self) {
        self.full_topic = build_subscribe_topic(&self.role, &self.plug_name, self.id.as_deref());
    }
}

impl TryFrom<&str> for ThreePartTopic {
    type Error = anyhow::Error;

    /// Try to convert a topic string into a valid Tether Three Part Topic
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let parts = value.split('/').collect::<Vec<&str>>();

        if parts.len() < 2 || parts.len() > 3 {
            return Err(anyhow!(
                "Did not find exactly 2 or 3 parts in the topic {}",
                value
            ));
        } else {
            debug!("parts: {:?}", parts);
        }

        let role = parts.first().expect("the role part should exist");

        let plug_name = parts.get(1).expect("the plug_name part should exist");

        match parts.get(2) {
            Some(id_part) => {
                if *id_part == "#" {
                    debug!("This must be a topic used for subscribing");
                    Ok(ThreePartTopic::new_for_subscribe(
                        plug_name,
                        Some(role),
                        Some("#"),
                    ))
                } else {
                    Ok(ThreePartTopic::new_three(role, plug_name, *id_part))
                }
            }
            None => Ok(ThreePartTopic::new_two(role, plug_name)),
        }
    }
}

pub fn build_subscribe_topic(role: &str, plug_name: &str, id: Option<&str>) -> String {
    match id {
        Some(id) => format!("{role}/{plug_name}/{id}"),
        None => format!("{role}/{plug_name}/#"),
    }
}

pub fn build_publish_topic(role: &str, plug_name: &str, id: Option<&str>) -> String {
    match id {
        Some(id) => format!("{role}/{plug_name}/{id}"),
        None => format!("{role}/{plug_name}"),
    }
}

pub fn parse_plug_name(topic: &str) -> Option<&str> {
    let parts: Vec<&str> = topic.split('/').collect();
    parts.get(1).copied()
}

pub fn parse_agent_id(topic: &str) -> Option<&str> {
    let parts: Vec<&str> = topic.split('/').collect();
    parts.get(2).copied()
}

pub fn parse_agent_role(topic: &str) -> Option<&str> {
    let parts: Vec<&str> = topic.split('/').collect();
    parts.first().copied()
}

#[cfg(test)]
mod tests {
    use crate::{
        three_part_topic::{parse_agent_id, parse_agent_role, parse_plug_name},
        TetherAgentOptionsBuilder,
    };

    use super::ThreePartTopic;

    #[test]
    fn util_parsers() {
        assert_eq!(parse_agent_role("one/two/three"), Some("one"));
        assert_eq!(parse_agent_id("one/two/three"), Some("three"));
        assert_eq!(parse_plug_name("one/two/three"), Some("two"));
        assert_eq!(parse_plug_name("just/two"), Some("two"));
    }

    #[test]
    fn build_full_topic() {
        let agent = TetherAgentOptionsBuilder::new("testingRole")
            .auto_connect(false)
            .build()
            .expect("failed to construct agent");
        let publishing_plug_topic = ThreePartTopic::new_for_publish(&agent, "testPlug", None, None);
        assert_eq!(&publishing_plug_topic.full_topic, "testingRole/testPlug/#");
    }
}
