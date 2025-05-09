use std::time::Duration;

use env_logger::{Builder, Env};
use log::{debug, info};
use serde::Serialize;
use tether_agent::*;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CustomStruct {
    id: usize,
    name: String,
}
fn main() {
    println!("Rust Tether Agent publish example");

    let mut builder = Builder::from_env(Env::default().default_filter_or("info"));
    builder.init();

    debug!("Debugging is enabled; could be verbose");

    let tether_agent = TetherAgentBuilder::new("rustExample")
        .build()
        .expect("failed to connect Tether");
    let AgentConfig { role, id, .. } = tether_agent.config();
    info!("Created agent OK: {}, {:?}", role, id.as_deref());

    let sender_definition =
        ChannelSenderDefBuilder::new("customStructs").build(tether_agent.config());
    let sender = tether_agent.create_sender_with_def::<CustomStruct>(sender_definition);

    // let test_struct = CustomStruct {
    //     id: 101,
    //     name: "something".into(),
    // };
    let payload = rmp_serde::to_vec_named(&101).expect("failed to serialize");
    sender
        .send_raw(&tether_agent, &payload)
        .expect("failed to send");

    let another_struct = CustomStruct {
        id: 202,
        name: "auto encoded".into(),
    };

    // Notice how the line below will produce a compiler error, whereas sender.send_raw for the
    // exact same payload (101) is fine, because .send_raw is not type-checked.
    // sender.send(&101).expect("failed to encode+send");

    sender
        .send(&tether_agent, &another_struct)
        .expect("failed to encode+send");

    let number_sender = tether_agent.create_sender::<u8>("numbersOnly");

    number_sender
        .send(&tether_agent, &8)
        .expect("failed to send");

    std::thread::sleep(Duration::from_millis(3000));
}
