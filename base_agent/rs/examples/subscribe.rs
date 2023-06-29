use std::{thread, time::Duration};

use env_logger::{Builder, Env};
use log::{debug, info, warn};
use rmp_serde::from_slice;
use serde::Deserialize;
use tether_agent::{PlugOptionsBuilder, TetherAgentOptionsBuilder};

#[derive(Deserialize, Debug)]
struct CustomMessage {
    id: usize,
    name: String,
}
// Test this by sending a message like
// tether-send --host localhost --topic test/any/two --message \{\"id\":1,\"name\":\"boo\"\}

fn main() {
    println!("Rust Tether Agent subscribe example");

    let mut builder = Builder::from_env(Env::default().default_filter_or("debug"));
    builder.init();

    debug!("Debugging is enabled; could be verbose");

    let tether_agent = TetherAgentOptionsBuilder::new("RustDemoAgent")
        .id("example")
        .build()
        .expect("failed to init Tether agent");

    let input_one = PlugOptionsBuilder::create_input("one").build(&tether_agent);
    debug!("input one {} = {}", input_one.name(), input_one.topic());
    let input_two = PlugOptionsBuilder::create_input("two").build(&tether_agent);
    debug!("input two {} = {}", input_two.name(), input_two.topic());
    let input_empty = PlugOptionsBuilder::create_input("nothing").build(&tether_agent);

    info!("Checking messages every 1s, 10x...");

    for i in 1..10 {
        info!("#{i}: Checking for messages...");
        while let Some((plug_name, message)) = tether_agent.check_messages() {
            debug!(
                "Received a message topic {} => plug name {}",
                message.topic(),
                plug_name
            );
            if &plug_name == input_one.name() {
                info!(
                    "******** INPUT ONE:\n Received a message from plug named \"{}\" on topic {} with length {} bytes",
                    input_one.name(),
                    message.topic(),
                    message.payload().len()
                );
            }
            if &plug_name == input_two.name() {
                info!(
                    "******** INPUT TWO:\n Received a message from plug named \"{}\" on topic {} with length {} bytes",
                    input_two.name(),
                    message.topic(),
                    message.payload().len()
                );
                // Notice how you must give the from_slice function a type so it knows what to expect
                let decoded = from_slice::<CustomMessage>(&message.payload());
                match decoded {
                    Ok(d) => {
                        info!("Yes, we decoded the MessagePack payload as: {:?}", d);
                        let CustomMessage { name, id } = d;
                        debug!("Name is {} and ID is {}", name, id);
                    }
                    Err(e) => {
                        warn!("Failed to decode the payload: {}", e)
                    }
                };
            }
            if &plug_name == input_empty.name() {
                info!(
                    "******** EMPTY MESSAGE:\n Received a message from plug named \"{}\" on topic {} with length {} bytes",
                    input_empty.name(),
                    message.topic(),
                    message.payload().len()
                );
            }
        }
        thread::sleep(Duration::from_millis(1000))
    }
}
