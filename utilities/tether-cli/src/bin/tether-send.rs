use cli_shared::defaults::AGENT_ROLE;
use env_logger::{Builder, Env};
use log::debug;
use serde::Serialize;
use tether_agent::{PlugOptionsBuilder, TetherAgentOptionsBuilder};

#[derive(Serialize)]
struct DummyData {
    id: usize,
    a_float: f32,
    an_int_array: Vec<usize>,
    a_string: String,
}

use clap::{command, Parser};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// Specify an Agent Role; this will be used for the auto-generated publish topic
    /// (ignored if you provide your own plug.topic)
    #[arg(long = "agent.role")]
    pub agent_role: Option<String>,

    /// Specify an Agent ID or Group; this will be used for the auto-generated publish topic
    /// (ignored if you provide your own plug.topic)
    #[arg(long = "agend.id")]
    pub agent_id: Option<String>,

    /// Overide the auto-generated topic with your own, to use with every published message
    #[arg(long = "plug.topic", default_value_t=String::from(format!("{}/any/testMessages", AGENT_ROLE)))]
    pub plug_topic: String,

    #[arg(long = "loglevel",default_value_t=String::from("info"))]
    pub log_level: String,
}

fn main() {
    let cli = Cli::parse();

    let mut builder = Builder::from_env(Env::default().default_filter_or(cli.log_level));
    builder.init();

    debug!("Debugging is enabled; could be verbose");

    let agent_role = match cli.agent_role {
        Some(custom) => custom,
        None => AGENT_ROLE.into(),
    };

    debug!("Using agent role = {}", &agent_role);

    let tether_agent = TetherAgentOptionsBuilder::new(&agent_role)
        .build()
        .expect("failed to connect Tether");

    let output = PlugOptionsBuilder::create_output("dummy").build(&tether_agent);

    let dummy_data = DummyData {
        id: 0,
        a_float: 42.0,
        an_int_array: vec![1, 2, 3, 4],
        a_string: "hello world".into(),
    };
    tether_agent
        .encode_and_publish(&output, &dummy_data)
        .expect("failed to publish");
}
