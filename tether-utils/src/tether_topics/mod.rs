use clap::Args;

pub mod agent_tree;
pub mod insights;
pub mod sampler;

#[derive(Args, Clone)]
pub struct TopicOptions {
    #[arg(long = "topic", default_value_t=String::from("#"))]
    pub topic: String,

    /// Sampler interval, in milliseconds
    #[arg(long = "sampler.interval", default_value_t = 1000)]
    pub sampler_interval: u64,

    /// Whether to print message rate and activity graph to the terminal;
    /// some terminals might break
    #[arg(long = "graph.enable")]
    pub graph_enable: bool,
}

impl Default for TopicOptions {
    fn default() -> Self {
        TopicOptions {
            topic: "#".into(),
            sampler_interval: 1000,
            graph_enable: false,
        }
    }
}

pub fn parse_channel_name(topic: &str) -> Option<&str> {
    let parts: Vec<&str> = topic.split('/').collect();
    match parts.get(1) {
        Some(s) => Some(*s),
        None => None,
    }
}

pub fn parse_agent_id(topic: &str) -> Option<&str> {
    let parts: Vec<&str> = topic.split('/').collect();
    match parts.get(2) {
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
