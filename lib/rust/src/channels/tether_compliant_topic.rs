use anyhow::anyhow;
use log::*;
use serde::{Deserialize, Serialize};

use crate::AgentConfig;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct TetherCompliantTopic {
    role: String,
    id: Option<String>,
    channel_name: String,
    full_topic: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum TetherOrCustomTopic {
    Tether(TetherCompliantTopic),
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

impl TetherCompliantTopic {
    /// Publish topics fall back to the ID and/or role associated with the agent, if not explicitly provided
    pub fn new_for_publish(
        agent_config: &AgentConfig,
        channel_name: &str,
        role_part_override: Option<&str>,
        id_part_override: Option<&str>,
    ) -> TetherCompliantTopic {
        let role = role_part_override.unwrap_or(&agent_config.role);
        let full_topic = build_publish_topic(role, channel_name, id_part_override);
        TetherCompliantTopic {
            role: role.into(),
            id: id_part_override.map(String::from),
            channel_name: channel_name.into(),
            full_topic,
        }
    }

    /// Subscribe topics fall back to wildcard `+` for role if not explicitly provided.
    pub fn new_for_subscribe(
        channel_name: &str,
        role_part_override: Option<&str>,
        id_part_override: Option<&str>,
    ) -> TetherCompliantTopic {
        let role = role_part_override.unwrap_or("+");
        let full_topic = build_subscribe_topic(role, channel_name, id_part_override);

        TetherCompliantTopic {
            role: role.into(),
            id: id_part_override.map(String::from),
            channel_name: channel_name.into(),
            full_topic,
        }
    }

    /// Directly constructs a Three Part Topic with explicitly provided role, channel_name, and id.
    pub fn new_three(role: &str, channel_name: &str, id: &str) -> TetherCompliantTopic {
        TetherCompliantTopic {
            role: role.into(),
            id: Some(id.into()),
            channel_name: channel_name.into(),
            full_topic: build_subscribe_topic(role, channel_name, Some(id)),
        }
    }

    /// Directly constructs a Three Part Topic with explicitly provided role, channel_name, and id.
    pub fn new_two(role: &str, channel_name: &str) -> TetherCompliantTopic {
        TetherCompliantTopic {
            role: role.into(),
            id: None,
            channel_name: channel_name.into(),
            full_topic: format!("{role}/{channel_name}"),
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

    pub fn channel_name(&self) -> &str {
        &self.channel_name
    }

    pub fn set_role(&mut self, role: &str) {
        self.role = role.into();
        self.update_full_topic();
    }

    pub fn set_id(&mut self, id: Option<&str>) {
        self.id = id.map(|id| id.into());
        self.update_full_topic();
    }

    pub fn set_channel_name(&mut self, channel_name: &str) {
        self.channel_name = channel_name.into();
        self.update_full_topic();
    }

    fn update_full_topic(&mut self) {
        self.full_topic = build_subscribe_topic(&self.role, &self.channel_name, self.id.as_deref());
    }
}

impl TryFrom<&str> for TetherCompliantTopic {
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

        let channel_name = parts.get(1).expect("the channel_name part should exist");

        match parts.get(2) {
            Some(id_part) => {
                if *id_part == "#" {
                    debug!("This must be a topic used for subscribing");
                    Ok(TetherCompliantTopic::new_for_subscribe(
                        channel_name,
                        Some(role),
                        Some("#"),
                    ))
                } else {
                    Ok(TetherCompliantTopic::new_three(role, channel_name, id_part))
                }
            }
            None => Ok(TetherCompliantTopic::new_two(role, channel_name)),
        }
    }
}

pub fn build_subscribe_topic(role: &str, channel_name: &str, id: Option<&str>) -> String {
    match id {
        Some(id) => format!("{role}/{channel_name}/{id}"),
        None => format!("{role}/{channel_name}/#"),
    }
}

pub fn build_publish_topic(role: &str, channel_name: &str, id: Option<&str>) -> String {
    match id {
        Some(id) => format!("{role}/{channel_name}/{id}"),
        None => format!("{role}/{channel_name}"),
    }
}

pub fn parse_channel_name(topic: &str) -> Option<&str> {
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
        agent::builder::TetherAgentBuilder,
        tether_compliant_topic::{parse_agent_id, parse_agent_role, parse_channel_name},
    };

    use super::TetherCompliantTopic;

    #[test]
    fn util_parsers() {
        assert_eq!(parse_agent_role("one/two/three"), Some("one"));
        assert_eq!(parse_agent_id("one/two/three"), Some("three"));
        assert_eq!(parse_channel_name("one/two/three"), Some("two"));
        assert_eq!(parse_channel_name("just/two"), Some("two"));
    }

    #[test]
    fn build_full_topic() {
        let agent = TetherAgentBuilder::new("testingRole")
            .auto_connect(false)
            .build()
            .expect("failed to construct agent");
        let publishing_chanel_topic =
            TetherCompliantTopic::new_for_publish(agent.config(), "testChannel", None, None);
        assert_eq!(
            &publishing_chanel_topic.full_topic,
            "testingRole/testChannel"
        );
    }
}
