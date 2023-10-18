use anyhow::anyhow;
use log::debug;
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
        role: Option<String>,
        id: Option<String>,
        plug_name: &str,
        agent: &TetherAgent,
    ) -> ThreePartTopic {
        let role = role.unwrap_or(agent.role().into());
        let id = id.unwrap_or(agent.id().into());
        let plug_name = String::from(plug_name);
        let full_topic = build_topic(&role, &id, &plug_name);
        ThreePartTopic {
            role,
            id,
            plug_name,
            full_topic,
        }
    }

    /// Subscribe topics fall back to wildcard `+` for role and/or id if not explicitly provided
    pub fn new_for_subscribe(
        plug_name: &str,
        role: Option<String>,
        id: Option<String>,
        plug_name_override: Option<String>,
    ) -> ThreePartTopic {
        let role = role.unwrap_or("+".into());
        let id = id.unwrap_or("+".into());
        let plug_name_part = match plug_name_override {
            Some(s) => s.clone(),
            None => String::from(plug_name),
        };
        let full_topic = build_topic(&role, &id, &plug_name_part);

        ThreePartTopic {
            role,
            id,
            plug_name: plug_name_part,
            full_topic,
        }
    }

    pub fn new(role: &str, id: &str, plug_name: &str) -> ThreePartTopic {
        ThreePartTopic {
            role: role.into(),
            id: id.into(),
            plug_name: plug_name.into(),
            full_topic: build_topic(&role, &id, &plug_name),
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

        let role = parts.get(0).expect("the role part should exist");
        let id = parts.get(1).expect("the id part should exist");
        let plug_name = parts.get(2).expect("the plug_name part should exist");

        return Ok(ThreePartTopic::new(role, id, plug_name));
    }
}

fn build_topic(role: &str, id: &str, plug_name: &str) -> String {
    format!("{role}/{id}/{plug_name}")
}
