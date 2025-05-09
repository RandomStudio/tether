use circular_buffer::CircularBuffer;
use tether_agent::{TetherAgent, tether_compliant_topic::TetherOrCustomTopic};

use crate::tether_topics::{agent_tree::AgentTree, sampler::Sampler};
use std::{
    fmt,
    time::{Duration, SystemTime},
};

use super::{TopicOptions, parse_agent_id, parse_agent_role, parse_channel_name};
pub const MONITOR_LOG_LENGTH: usize = 256;

/// Topic, Payload as JSON
type MessageLogEntry = (String, String);

pub struct Insights {
    topics: Vec<String>,
    roles: Vec<String>,
    ids: Vec<String>,
    channels: Vec<String>,
    trees: Vec<AgentTree>,
    message_count: u128,
    log_start: Option<SystemTime>,
    message_log: CircularBuffer<MONITOR_LOG_LENGTH, MessageLogEntry>,
    sampler: Sampler,
}

impl fmt::Display for Insights {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let topics = format!("x{} Topics: {:?} \n\n", self.topics().len(), self.topics());
        let roles = format!("x{} Roles: {:?} \n", self.roles().len(), self.roles());
        let ids = format!("x{} IDs: {:?} \n", self.ids().len(), self.ids());
        let channels = format!(
            "x{} Channels: {:?} \n",
            self.channels().len(),
            self.channels()
        );

        let trees_formatted = self.trees.iter().map(|x| x.to_string()).collect::<String>();

        write!(
            f,
            "{}{}{}{}{}",
            topics, roles, ids, channels, trees_formatted
        )
    }
}

impl Insights {
    pub fn new(options: &TopicOptions, tether_agent: &mut TetherAgent) -> Self {
        if !tether_agent.is_connected() {
            panic!("Insights utility needs already-connected Tether Agent");
        }

        if let Some(client) = tether_agent.client_mut() {
            client
                .subscribe(options.topic.clone(), tether_agent::mqtt::QoS::ExactlyOnce)
                .expect("failed to subscribe");
        } else {
            panic!("No client to subscribe to!")
        }

        // let _channel = ChannelOptionsBuilder::create_receiver("monitor")
        //     .topic(Some(options.topic.clone()).as_deref())
        //     .build(tether_agent)
        //     .expect("failed to connect Tether");

        Insights {
            topics: Vec::new(),
            roles: Vec::new(),
            ids: Vec::new(),
            channels: Vec::new(),
            trees: Vec::new(),
            message_log: CircularBuffer::new(),
            message_count: 0,
            log_start: None,
            sampler: Sampler::new(options.sampler_interval),
        }
    }

    pub fn sample(&mut self) -> bool {
        self.sampler.add_sample(self.message_count)
    }

    pub fn sampler(&self) -> &Sampler {
        &self.sampler
    }

    pub fn update(&mut self, topic: &TetherOrCustomTopic, payload: Vec<u8>) -> bool {
        self.message_count += 1;

        if self.log_start.is_none() {
            self.log_start = Some(SystemTime::now());
        }

        let full_topic_string = topic.full_topic_string();

        if payload.is_empty() {
            self.message_log
                .push_back((String::from(&full_topic_string), "[EMPTY_MESSAGE]".into()));
        } else {
            let value: rmpv::Value =
                rmp_serde::from_slice(&payload).expect("failed to decode msgpack");
            let json = serde_json::to_string(&value).expect("failed to stringify JSON");
            self.message_log
                .push_back((String::from(&full_topic_string), json));
        }

        let mut did_change = false;

        // Collect some stats...
        if add_if_unique(&full_topic_string, &mut self.topics) {
            did_change = true;
        }
        if add_if_unique(
            parse_agent_role(&full_topic_string).unwrap_or("unknown"),
            &mut self.roles,
        ) {
            did_change = true;
        }
        if add_if_unique(
            parse_agent_id(&full_topic_string).unwrap_or("unknown"),
            &mut self.ids,
        ) {
            did_change = true;
        }
        if add_if_unique(
            parse_channel_name(&full_topic_string).unwrap_or("unknown"),
            &mut self.channels,
        ) {
            did_change = true;
        }

        if did_change {
            self.trees = self
                .roles()
                .iter()
                .map(|role| AgentTree::new(role.as_str(), self.topics()))
                .collect::<Vec<AgentTree>>();
        }

        did_change
    }

    pub fn topics(&self) -> &[String] {
        &self.topics
    }

    pub fn roles(&self) -> &[String] {
        &self.roles
    }
    pub fn ids(&self) -> &[String] {
        &self.ids
    }
    pub fn channels(&self) -> &[String] {
        &self.channels
    }
    pub fn trees(&self) -> &[AgentTree] {
        &self.trees
    }

    pub fn message_count(&self) -> u128 {
        self.message_count
    }

    pub fn message_log(&self) -> &CircularBuffer<MONITOR_LOG_LENGTH, MessageLogEntry> {
        &self.message_log
    }
    pub fn since_log_start(&self) -> Option<Duration> {
        self.log_start
            .map(|t| t.elapsed().unwrap_or(Duration::ZERO))
    }

    /// Messages per second, since log_start was (re)set
    pub fn get_rate(&self) -> Option<f32> {
        match self.log_start {
            Some(t) => {
                if let Ok(elapsed) = t.elapsed() {
                    Some(self.message_count as f32 / elapsed.as_secs_f32())
                } else {
                    None
                }
            }
            None => None,
        }
    }
}

fn add_if_unique(item: &str, list: &mut Vec<String>) -> bool {
    if !list.iter().any(|i| i.eq(item)) {
        list.push(String::from(item));
        true
    } else {
        false
    }
}
