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
}

impl Default for TopicOptions {
    fn default() -> Self {
        TopicOptions {
            topic: "#".into(),
            sampler_interval: 1000,
        }
    }
}
