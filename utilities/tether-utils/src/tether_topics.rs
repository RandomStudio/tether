use clap::Args;
use log::info;
use tether_agent::{
    mqtt::Message, parse_agent_id, parse_agent_role, parse_plug_name, PlugOptionsBuilder,
    TetherAgent,
};

#[derive(Args)]
pub struct TopicOptions {
    #[arg(long = "topic", default_value_t=String::from("#"))]
    subscribe_topic: String,
}

#[derive(Debug)]
pub struct Insights {
    topics: Vec<String>,
    roles: Vec<String>,
    ids: Vec<String>,
    plugs: Vec<String>,
    // message_count: u64,
}

impl Insights {
    fn new() -> Self {
        Insights {
            topics: Vec::new(),
            roles: Vec::new(),
            ids: Vec::new(),
            plugs: Vec::new(),
            // message_count: 0,
        }
    }

    pub fn update(&mut self, _plug_name: &str, message: &Message) -> bool {
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
}

fn add_if_unique(item: &str, list: &mut Vec<String>) -> bool {
    if !list.iter().any(|i| i.eq(item)) {
        list.push(String::from(item));
        true
    } else {
        false
    }
}

pub fn topics(options: &TopicOptions, tether_agent: &TetherAgent) {
    info!("Tether Topics Parsing Utility");

    let _input = PlugOptionsBuilder::create_input("all")
        .topic(&options.subscribe_topic)
        .build(tether_agent);

    let mut insights = Insights::new();

    loop {
        let mut did_work = false;
        while let Some((plug_name, message)) = tether_agent.check_messages() {
            did_work = true;
            if insights.update(&plug_name, &message) {
                info!("{:#?}", insights);
            }
        }
        if !did_work {
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
    }
}
