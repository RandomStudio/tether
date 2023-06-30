use env_logger::{Builder, Env};
use log::{debug, info};
use tether_agent::{PlugOptionsBuilder, TetherAgentOptionsBuilder};

fn main() -> ! {
    let mut builder = Builder::from_env(Env::default().default_filter_or("info"));
    builder.init();

    debug!("Debugging is enabled; could be verbose");

    let tether_agent = TetherAgentOptionsBuilder::new("testAgent")
        .build()
        .expect("failed to connect Tether");

    let _input = PlugOptionsBuilder::create_input("all")
        .topic("#")
        .build(&tether_agent);

    loop {
        while let Some((plug_name, message)) = tether_agent.check_messages() {
            debug!("Received message on plug {}: {:?}", plug_name, message);
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
