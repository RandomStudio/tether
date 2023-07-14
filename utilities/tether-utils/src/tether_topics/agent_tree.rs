use std::fmt;
use tether_agent::{parse_agent_id, parse_plug_name};

/// Role, IDs, OutputPlugs
pub struct AgentTree {
    pub role: String,
    pub ids: Vec<String>,
    pub output_plugs: Vec<String>,
}

impl AgentTree {
    pub fn new(role: &str, topics: &[String]) -> AgentTree {
        let topics_this_agent = topics.iter().filter_map(|topic| {
            if topic.contains(role) {
                Some(String::from(topic))
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

        let output_plugs = topics_this_agent
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
            output_plugs: output_plugs.to_vec(),
        }
    }
}

impl fmt::Display for AgentTree {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Self {
            role,
            ids,
            output_plugs,
        } = self;
        let ids_list = ids
            .iter()
            .fold(String::from(""), |acc, x| format!("{}\n    - {}", acc, x));
        let output_plugs_list = output_plugs.iter().fold(String::from(""), |acc, x| {
            format!("{}\n        - {}", acc, x)
        });
        write!(f, "\n{}\n {}\n {}\n", role, ids_list, output_plugs_list)
    }
}
