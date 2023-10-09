use crate::*;
use log::{debug, error, warn};
use serde::{Deserialize, Serialize};

pub mod definitions;
pub mod options;
pub mod three_part_topic;

pub use definitions::*;
pub use options::*;

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

// pub fn default_subscribe_topic(plug_name: &str) -> String {
//     format!("+/+/{plug_name}")
// }

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
