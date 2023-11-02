use anyhow::anyhow;
use log::{debug, error};
use serde::{Deserialize, Serialize};

use crate::TetherAgent;

#[derive(Serialize, Deserialize, Debug)]
pub struct ThreePartTopic {
    role: String,
    id: String,
    plug_name: String,
    full_topic: String,
}

impl ThreePartTopic {
    /// Publish topics fall back to the ID and/or role associated with the agent, if not explicitly provided
    pub fn new_for_publish(
        role: Option<&str>,
        id: Option<&str>,
        plug_name: &str,
        agent: &TetherAgent,
    ) -> ThreePartTopic {
        let role = role.unwrap_or(agent.role());
        let id = id.unwrap_or(agent.id());
        let full_topic = build_topic(role, id, plug_name);
        ThreePartTopic {
            role: role.into(),
            id: id.into(),
            plug_name: plug_name.into(),
            full_topic,
        }
    }

    /// Subscribe topics fall back to wildcard `+` for role and/or id if not explicitly provided.
    /// If `plug_name_part` is specified as `Some(String)` then the plug name part of the generated
    /// topic is changed but the plug name itself is left alone.
    pub fn new_for_subscribe(
        plug_name: &str,
        role: Option<&str>,
        id: Option<&str>,
        plug_name_part_override: Option<&str>,
    ) -> ThreePartTopic {
        let role = role.unwrap_or("+");
        let id = id.unwrap_or("+");
        let plug_name_part = match plug_name_part_override {
            Some(s) => {
                if !&s.eq("+") {
                    error!("The only valid override for the Plug Name part is a wildcard (+)");
                }
                s
            }
            None => plug_name,
        };
        let full_topic = build_topic(role, id, plug_name_part);

        ThreePartTopic {
            role: role.into(),
            id: id.into(),
            plug_name: plug_name_part.into(),
            full_topic,
        }
    }

    pub fn new(role: &str, id: &str, plug_name: &str) -> ThreePartTopic {
        ThreePartTopic {
            role: role.into(),
            id: id.into(),
            plug_name: plug_name.into(),
            full_topic: build_topic(role, id, plug_name),
        }
    }

    pub fn topic(&self) -> &str {
        &self.full_topic
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
        self.update_full_topic();
    }

    pub fn set_id(&mut self, id: &str) {
        self.id = id.into();
        self.update_full_topic();
    }

    pub fn set_plug_name(&mut self, plug_name: &str) {
        self.plug_name = plug_name.into();
        self.update_full_topic();
    }

    fn update_full_topic(&mut self) {
        self.full_topic = build_topic(&self.role, &self.id, &self.plug_name);
    }
}

impl TryFrom<&str> for ThreePartTopic {
    type Error = anyhow::Error;

    /// Try to convert a topic string into a valid Tether Three Part Topic
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

        let role = parts.first().expect("the role part should exist");
        let id = parts.get(1).expect("the id part should exist");
        let plug_name = parts.get(2).expect("the plug_name part should exist");

        Ok(ThreePartTopic::new(role, id, plug_name))
    }
}

pub fn build_topic(role: &str, id: &str, plug_name: &str) -> String {
    format!("{role}/{id}/{plug_name}")
}

pub fn parse_plug_name(topic: &str) -> Option<&str> {
    let parts: Vec<&str> = topic.split('/').collect();
    match parts.get(2) {
        Some(s) => Some(*s),
        None => None,
    }
}

pub fn parse_agent_id(topic: &str) -> Option<&str> {
    let parts: Vec<&str> = topic.split('/').collect();
    match parts.get(1) {
        Some(s) => Some(*s),
        None => None,
    }
}

pub fn parse_agent_role(topic: &str) -> Option<&str> {
    let parts: Vec<&str> = topic.split('/').collect();
    match parts.first() {
        Some(s) => Some(*s),
        None => None,
    }
}

#[cfg(test)]
mod tests {
    use crate::three_part_topic::{parse_agent_id, parse_agent_role, parse_plug_name};

    #[test]
    fn util_parsers() {
        assert_eq!(parse_agent_role("one/two/three"), Some("one"));
        assert_eq!(parse_agent_id("one/two/three"), Some("two"));
        assert_eq!(parse_plug_name("one/two/three"), Some("three"));
        assert_eq!(parse_plug_name("just/two"), None);
    }
}
