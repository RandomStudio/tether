use clap::Args;

pub mod agent_tree;
pub mod insights;

#[derive(Args, Clone)]
pub struct TopicOptions {
    #[arg(long = "topic", default_value_t=String::from("#"))]
    pub topic: String,
}

impl Default for TopicOptions {
    fn default() -> Self {
        TopicOptions { topic: "#".into() }
    }
}
