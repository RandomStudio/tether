use std::time::Duration;

use env_logger::{Builder, Env};
use log::{debug, info};
use serde::Serialize;
use tether_agent::{ChannelOptionsBuilder, TetherAgentOptionsBuilder};

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

    let tether_agent = TetherAgentOptionsBuilder::new("rustExample")
        .build()
        .expect("failed to connect Tether");
    let (role, id, _) = tether_agent.description();
    info!("Created agent OK: {}, {}", role, id);

    let sender = tether_agent.create_sender("values");

    let test_struct = CustomStruct {
        id: 101,
        name: "something".into(),
    };
    let payload = rmp_serde::to_vec_named(&test_struct).expect("failed to serialize");
    sender.send_raw(&payload).expect("failed to send");

    let another_struct = CustomStruct {
        id: 202,
        name: "auto encoded".into(),
    };

    sender.send(&another_struct).expect("failed to encode+send");

    std::thread::sleep(Duration::from_millis(3000));
}
