use clap::Args;
use log::*;
use tether_agent::{
    mqtt::Message, parse_agent_id, parse_agent_role, parse_plug_name, PlugDefinition,
    PlugOptionsBuilder, TetherAgent,
};

#[derive(Args)]
pub struct TopicOptions {
    #[arg(long = "topic", default_value_t=String::from("#"))]
    pub subscribe_topic: String,
}

#[derive(Debug)]
pub struct Insights {
    pub topics: Vec<String>,
    pub roles: Vec<String>,
    pub ids: Vec<String>,
    pub plugs: Vec<String>,
    // message_count: u64,
}

impl Insights {
    pub fn new() -> Self {
        Insights {
            topics: Vec::new(),
            roles: Vec::new(),
            ids: Vec::new(),
            plugs: Vec::new(),
            // message_count: 0,
        }
    }

    pub fn update(&mut self, message: &Message) -> bool {
        // self.message_count += 1;
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

    pub fn check_for_updates(
        &mut self,
        input_plug_subscribed: &PlugDefinition,
        tether_agent: &TetherAgent,
    ) -> bool {
        match input_plug_subscribed {
            PlugDefinition::InputPlugDefinition(_) => {
                debug!("Input Plugs are ok; make sure you have subscribed")
            }
            _ => panic!("You should have created an Input Plug"),
        };
        let mut did_work = false;
        while let Some((_plug_name, message)) = tether_agent.check_messages() {
            did_work = true;
            if self.update(&message) {
                debug!("Insights update: {:#?}", self);
                return true;
            }
        }
        if !did_work {
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
        false
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

pub fn subscribe(
    options: &TopicOptions,
    tether_agent: &TetherAgent,
) -> anyhow::Result<PlugDefinition> {
    info!("Tether Topics Parsing Utility");

    match PlugOptionsBuilder::create_input("all")
        .topic(&options.subscribe_topic)
        .build(tether_agent)
    {
        Ok(p) => Ok(p),
        Err(e) => Err(e.into()),
    }
}
