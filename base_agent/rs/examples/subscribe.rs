use std::{thread, time::Duration};

use env_logger::{Builder, Env};
use log::{debug, info, warn};
use rmp_serde::from_slice;
use serde::Deserialize;
use tether_agent::TetherAgent;

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
                info!(
                    "******** INPUT ONE:\n Received a message from plug named \"{}\" on topic {} with length {} bytes",
                    input_one.name,
                    message.topic(),
                    message.payload().len()
                );
            }
            if &input_two.name == plug_name.as_str() {
                info!(
                    "******** INPUT TWO:\n Received a message from plug named \"{}\" on topic {} with length {} bytes",
                    input_two.name,
                    message.topic(),
                    message.payload().len()
                );
                // Notice how you must give the from_slice function a type so it knows what to expect
                let decoded = from_slice::<CustomMessage>(&message.payload());
                match decoded {
                    Ok(d) => { info!("Yes, we decoded the MessagePack payload as: {:?}", d)},
                    Err(e) => { warn!("Failed to decode the payload: {}", e)}
                };
            }
            if &input_empty.name == plug_name.as_str() {
                info!(
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
