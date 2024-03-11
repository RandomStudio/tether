use std::{thread, time::Duration};

use env_logger::{Builder, Env};
use log::{debug, info, warn};
use rmp_serde::from_slice;
use serde::Deserialize;

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

    let mut tether_agent = TetherAgentOptionsBuilder::new("RustDemoSubscribe")
        .id(Some("example"))
        .build()
        .expect("failed to init Tether agent");

    let input_one = PlugOptionsBuilder::create_input("one")
        .build(&mut tether_agent)
        .expect("failed to create input");
    debug!("input one {} = {}", input_one.name(), input_one.topic());
    let input_two = PlugOptionsBuilder::create_input("two")
        .role(Some("specific"))
        .build(&mut tether_agent)
        .expect("failed to create input");
    debug!("input two {} = {}", input_two.name(), input_two.topic());
    let input_empty = PlugOptionsBuilder::create_input("nothing")
        .build(&mut tether_agent)
        .expect("failed to create input");

    let input_everything = PlugOptionsBuilder::create_input("everything")
        .topic(Some("#"))
        .build(&mut tether_agent)
        .expect("failed to create input");

    let input_specify_id = PlugOptionsBuilder::create_input("groupMessages")
        .id(Some("someGroup"))
        .name(None)
        .build(&mut tether_agent)
        .expect("failed to create input");

    debug!(
        "input everything {} = {}",
        input_everything.name(),
        input_everything.topic()
    );

    info!("Checking messages every 1s, 10x...");

    for i in 1..10 {
        info!("#{i}: Checking for messages...");
        while let Some((topic, payload)) = tether_agent.check_messages() {
            // debug!(
            //     "........ Received a message topic {:?} => topic parts {:?}",
            //     topic, topic
            // );

            if input_one.matches(&topic) {
                info!(
                            "******** INPUT ONE:\n Received a message for plug named \"{}\" on topic {:?} with length {} bytes",
                            input_one.name(),
                            topic,
                            payload.len()
                        );
                // assert_eq!(parse_plug_name(topic.un), Some("one"));
            }
            if input_two.matches(&topic) {
                info!(
                        "******** INPUT TWO:\n Received a message for plug named \"{}\" on topic {:?} with length {} bytes",
                        input_two.name(),
                        topic,
                        payload.len()
                    );
                // assert_eq!(parse_plug_name(message.topic()), Some("two"));
                // assert_ne!(parse_plug_name(message.topic()), Some("one"));

                // Notice how you must give the from_slice function a type so it knows what to expect
                let decoded = from_slice::<CustomMessage>(&payload);
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
            if input_empty.matches(&topic) {
                info!(
                        "******** EMPTY MESSAGE:\n Received a message for plug named \"{}\" on topic {:?} with length {} bytes",
                        input_empty.name(),
                        topic,
                       payload.len()
                    );
                // assert_eq!(parse_plug_name(topic), Some("nothing"));
            }
            if input_everything.matches(&topic) {
                info!(
                    "******** EVERYTHING MATCHES HERE:\n Received a message for plug named \"{}\" on topic {:?} with length {} bytes",
                    input_everything.name(),
                    topic,
                   payload.len()
                );
            }
            if input_specify_id.matches(&topic) {
                info!("******** ID MATCH:\n Should match any role and plug name, but only messages with ID \"groupMessages\"");
                info!(
                    "\n Received a message from plug named \"{}\" on topic {:?} with length {} bytes",
                    input_specify_id.name(),
                    topic,
                    payload.len()
                );
                // assert_eq!(parse_agent_id(message.topic()), Some("groupMessages"));
            }
        }

        thread::sleep(Duration::from_millis(1000))
    }
}
