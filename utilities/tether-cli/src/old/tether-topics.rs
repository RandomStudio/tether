use cli_shared::defaults::AGENT_ROLE;
use env_logger::{Builder, Env};
use log::{debug, info};
use tether_agent::{
    mqtt::Message, parse_agent_id, parse_agent_role, parse_plug_name, PlugOptionsBuilder,
    TetherAgentOptionsBuilder,
};

use clap::{command, Parser};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[arg(long = "tether.host", default_value_t=String::from("localhost"))]
    pub tether_host: String,

    #[arg(long = "tether.port", default_value_t = 1883)]
    pub tether_port: u16,

    #[arg(long = "tether.username", default_value_t=String::from("tether"))]
    pub tether_username: String,

    #[arg(long = "tether.password", default_value_t=String::from("sp_ceB0ss!"))]
    pub tether_password: String,

    #[arg(long = "topic", default_value_t=String::from("#"))]
    pub subscribe_topic: String,

    #[arg(long = "loglevel",default_value_t=String::from("info"))]
    pub log_level: String,
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

    pub fn update(&mut self, _plug_name: &str, message: &Message) {
        // self.message_count += 1;

        // Collect some stats...
        add_if_unique(message.topic(), &mut self.topics);
        add_if_unique(
            parse_agent_role(message.topic()).unwrap_or("unknown"),
            &mut self.roles,
        );
        add_if_unique(
            parse_agent_id(message.topic()).unwrap_or("unknown"),
            &mut self.ids,
        );
        add_if_unique(
            parse_plug_name(message.topic()).unwrap_or("unknown"),
            &mut self.plugs,
        );
    }
}

fn add_if_unique(item: &str, list: &mut Vec<String>) {
    if !list.iter().any(|i| i.eq(item)) {
        list.push(String::from(item));
    }
}
fn main() {
    let cli = Cli::parse();

    let mut builder = Builder::from_env(Env::default().default_filter_or(cli.log_level));
    builder.init();

    debug!("Debugging is enabled; could be verbose");

    let tether_agent = TetherAgentOptionsBuilder::new(AGENT_ROLE)
        .host(&cli.tether_host)
        .port(cli.tether_port)
        .username(&cli.tether_username)
        .password(&cli.tether_password)
        .build()
        .expect("failed to connect Tether");

    let _input = PlugOptionsBuilder::create_input("all")
        .topic(&cli.subscribe_topic)
        .build(&tether_agent);

    let mut insights = Insights::new();

    loop {
        while let Some((plug_name, message)) = tether_agent.check_messages() {
            insights.update(&plug_name, &message);
            info!("{:#?}", insights);
        }
    }
}
