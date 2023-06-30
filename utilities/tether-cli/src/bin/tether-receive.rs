use cli_shared::defaults::AGENT_ROLE;
use env_logger::{Builder, Env};
use log::{debug, info};
use tether_agent::{PlugOptionsBuilder, TetherAgentOptionsBuilder};

use clap::{command, Parser};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[arg(long = "topic", default_value_t=String::from("#"))]
    pub subscribe_topic: String,

    #[arg(long = "loglevel",default_value_t=String::from("info"))]
    pub log_level: String,
}

fn main() -> ! {
    let cli = Cli::parse();

    let mut builder = Builder::from_env(Env::default().default_filter_or(cli.log_level));
    builder.init();

    debug!("Debugging is enabled; could be verbose");

    let tether_agent = TetherAgentOptionsBuilder::new(AGENT_ROLE)
        .build()
        .expect("failed to connect Tether");

    let _input = PlugOptionsBuilder::create_input("all")
        .topic(&cli.subscribe_topic)
        .build(&tether_agent);

    loop {
        while let Some((plug_name, message)) = tether_agent.check_messages() {
            debug!("Received message on plug {}: {:?}", plug_name, message);
            info!("Received message on topic \"{}\"", message.topic());
            let bytes = message.payload();
            if bytes.is_empty() {
                info!("Empty message payload");
            } else {
                let value: rmpv::Value =
                    rmp_serde::from_slice(bytes).expect("failed to decode msgpack");
                let json = serde_json::to_string(&value).expect("failed to stringify JSON");
                info!("Decoded MessagePack payload: {}", json);
            }
        }
    }
}
