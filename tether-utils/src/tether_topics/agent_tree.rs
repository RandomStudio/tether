use std::fmt;

use super::{parse_agent_id, parse_agent_role, parse_plug_name};

/// Role, IDs, Channels
pub struct AgentTree {
    pub role: String,
    pub ids: Vec<String>,
    pub channels: Vec<String>,
}

impl AgentTree {
    pub fn new(role: &str, topics: &[String]) -> AgentTree {
        let topics_this_agent = topics.iter().filter_map(|topic| {
            if let Some(role_part) = parse_agent_role(topic.as_str()) {
                if role_part == role {
                    Some(String::from(topic))
                } else {
                    None
                }
            } else {
                None
            }
        });

        let ids = topics_this_agent
            .clone()
            .fold(Vec::new(), |mut acc, topic| {
                if let Some(id) = parse_agent_id(&topic) {
                    if !acc.iter().any(|x| x == id) {
                        acc.push(String::from(id))
                    }
                }
                acc
            });

        let channels = topics_this_agent
            .clone()
            .fold(Vec::new(), |mut acc, topic| {
                if let Some(p) = parse_plug_name(&topic) {
                    acc.push(String::from(p));
                }
                acc
            });

        AgentTree {
            role: role.into(),
            ids: ids.to_vec(),
            channels: channels.to_vec(),
        }
    }
}

impl fmt::Display for AgentTree {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Self {
            role,
            ids,
            channels,
        } = self;
        let ids_list = ids
            .iter()
            .fold(String::from(""), |acc, x| format!("{}\n    - {}", acc, x));
        let channels_list = channels.iter().fold(String::from(""), |acc, x| {
            format!("{}\n        - {}", acc, x)
        });
        write!(f, "\n{}\n {}\n {}\n", role, ids_list, channels_list)
    }
}
