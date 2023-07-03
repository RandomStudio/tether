use env_logger::{Builder, Env};
use log::{debug, info};
use tether_agent::{PlugOptionsBuilder, TetherAgentOptionsBuilder};

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(long = "tether.host", default_value_t=String::from("localhost"))]
    pub tether_host: String,

    #[arg(long = "tether.port", default_value_t = 1883)]
    pub tether_port: u16,

    #[arg(long = "tether.username", default_value_t=String::from("tether"))]
    pub tether_username: String,

    #[arg(long = "tether.password", default_value_t=String::from("sp_ceB0ss!"))]
    pub tether_password: String,

    #[arg(long = "loglevel",default_value_t=String::from("info"))]
    pub log_level: String,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Receive {
        #[arg(long = "topic", default_value_t=String::from("#"))]
        subscribe_topic: String,
    },
    Send {
        /// Specify an Agent Role; this will be used for the auto-generated publish topic
        /// (ignored if you provide your own plug.topic)
        #[arg(long = "agent.role", default_value_t=String::from(defaults::AGENT_ROLE))]
        agent_role: String,

        /// Specify an Agent ID or Group; this will be used for the auto-generated publish topic
        /// (ignored if you provide your own plug.topic)
        #[arg(long = "agent.id", default_value_t=String::from(defaults::AGENT_ID))]
        agent_id: String,

        /// Overide the auto-generated topic with your own, to use with every published message
        #[arg(long = "plug.name", default_value_t=String::from("testMessages"))]
        plug_name: String,

        /// Overide the auto-generated topic with your own, to use with every published message
        #[arg(long = "plug.topic")]
        plug_topic: Option<String>,

        /// Optionally provide a custom message. Provide this as a valid JSON string.
        #[arg(long = "message")]
        custom_message: Option<String>,
    },
}

mod defaults {
    pub const AGENT_ROLE: &str = "testAgent";
    pub const AGENT_ID: &str = "any";
}

fn main() {
    let cli = Cli::parse();

    println!("Tether CLI; mode {:?}", cli.command);
}
