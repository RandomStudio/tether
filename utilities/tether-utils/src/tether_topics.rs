use std::fmt;

use circular_buffer::CircularBuffer;
use clap::Args;
use log::*;
use tether_agent::{
    mqtt::Message, parse_agent_id, parse_agent_role, parse_plug_name, PlugDefinition,
    PlugOptionsBuilder, TetherAgent,
};

#[derive(Args, Clone)]
pub struct TopicOptions {
    #[arg(long = "topic", default_value_t=String::from("#"))]
    pub subscribe_topic: String,
}

pub const MONITOR_LOG_LENGTH: usize = 256;
type MessageLogEntry = (String, String);

pub struct Insights {
    options: TopicOptions,
    topics: Vec<String>,
    roles: Vec<String>,
    ids: Vec<String>,
    plugs: Vec<String>,
    message_count: u128,
    message_log: CircularBuffer<MONITOR_LOG_LENGTH, MessageLogEntry>,
    input_plug_subscribed: Option<PlugDefinition>,
}

impl fmt::Display for Insights {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let topics = format!("x{} Topics: {:?} \n", self.topics().len(), self.topics());
        let roles = format!("x{} Roles: {:?} \n", self.roles().len(), self.roles());
        let ids = format!("x{} IDs: {:?} \n", self.ids().len(), self.ids());
        let plugs = format!("x{} Plugs: {:?} \n", self.plugs().len(), self.plugs());
        let message_count = format!("x{} Messages total \n", self.message_count());
        write!(f, "{}{}{}{}{}", topics, roles, ids, plugs, message_count)
    }
}

impl Insights {
    pub fn new(options: &TopicOptions) -> Self {
        Insights {
            options: options.clone(),
            topics: Vec::new(),
            roles: Vec::new(),
            ids: Vec::new(),
            plugs: Vec::new(),
            message_log: CircularBuffer::new(),
            message_count: 0,
            input_plug_subscribed: None,
        }
    }

    pub fn update(&mut self, message: &Message) -> bool {
        self.message_count += 1;
        let mut did_change = false;

        // Collect some stats...
        if add_if_unique(message.topic(), &mut self.topics) {
            did_change = true;
        }
        if add_if_unique(
            parse_agent_role(message.topic()).unwrap_or("unknown"),
            &mut self.roles,
        ) {
            did_change = true;
        }
        if add_if_unique(
            parse_agent_id(message.topic()).unwrap_or("unknown"),
            &mut self.ids,
        ) {
            did_change = true;
        }
        if add_if_unique(
            parse_plug_name(message.topic()).unwrap_or("unknown"),
            &mut self.plugs,
        ) {
            did_change = true;
        }

        did_change
    }

    pub fn check_for_updates(&mut self, tether_agent: &TetherAgent) -> bool {
        let mut did_work = false;

        match &self.input_plug_subscribed {
            None => {
                debug!("Subscribe for the first time");
                let input_plug = PlugOptionsBuilder::create_input("monitor")
                    .topic(&self.options.subscribe_topic)
                    .build(tether_agent)
                    .expect("failed to connect Tether");
                self.input_plug_subscribed = Some(input_plug);
            }
            Some(_input_plug) => {
                // debug!("Checking...");
                while let Some((_plug_name, message)) = tether_agent.check_messages() {
                    debug!("Got message on topic \"{}\"", message.topic());
                    did_work = true;
                    if self.update(&message) {
                        debug!("Insights update");
                        return true;
                    }
                }
            }
        }

        if !did_work {
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
        false
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
    pub fn plugs(&self) -> &[String] {
        &self.plugs
    }

    pub fn message_count(&self) -> u128 {
        self.message_count
    }

    pub fn message_log(&self) -> &CircularBuffer<MONITOR_LOG_LENGTH, MessageLogEntry> {
        &self.message_log
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
