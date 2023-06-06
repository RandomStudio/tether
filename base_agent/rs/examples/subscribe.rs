use std::{thread, time::Duration};

use env_logger::{Builder, Env};
use log::{debug, info};
use tether_agent::TetherAgent;

fn main() {
    println!("Rust Tether Agent subscribe example");

    let mut builder = Builder::from_env(Env::default().default_filter_or("debug"));
    builder.init();

    debug!("Debugging is enabled; could be verbose");

    let agent = TetherAgent::new("RustDemoAgent", Some("example"), None);

    agent.connect(None, None).expect("Failed to connect");

    let input_one = agent.create_input_plug("one", None, None).unwrap();
    let input_two = agent.create_input_plug("two", None, None).unwrap();
    let input_empty = agent.create_input_plug("nothing", None, None).unwrap();

    info!("Checking messages every 1s, 10x...");

    for i in 1..10 {
        info!("#{i}: Checking for messages...");
        if let Some((plug_name, message)) = agent.check_messages() {
            if &input_one.name == plug_name.as_str() {
                println!(
                    "******** INPUT ONE:\n Received a message from plug named \"{}\" on topic {} with length {} bytes",
                    input_one.name,
                    message.topic(),
                    message.payload().len()
                );
            }
            if &input_two.name == plug_name.as_str() {
                println!(
                    "******** INPUT TWO:\n Received a message from plug named \"{}\" on topic {} with length {} bytes",
                    input_two.name,
                    message.topic(),
                    message.payload().len()
                );
            }
            if &input_empty.name == plug_name.as_str() {
                println!(
                    "******** EMPTY MESSAGE:\n Received a message from plug named \"{}\" on topic {} with length {} bytes",
                    input_empty.name,
                    message.topic(),
                    message.payload().len()
                );
            }
        }
        thread::sleep(Duration::from_millis(1000))
    }
}
