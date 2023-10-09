use env_logger::{Builder, Env};
use log::*;
use serde::Serialize;
use tether_agent::{PlugOptionsBuilder, TetherAgentOptionsBuilder};

#[derive(Serialize)]
struct CustomStruct {
    id: usize,
    name: String,
}

fn main() {
    let mut builder = Builder::from_env(Env::default().default_filter_or("info"));
    builder.init();

    debug!("Debugging is enabled; could be verbose");

    let bad_tether_agent = TetherAgentOptionsBuilder::new("tester")
        .host("tether-io.dev")
        .username("bla")
        .password("bla")
        .build();
    match bad_tether_agent {
        Ok(_agent) => {
            panic!("Connection: This shouldn't work!");
        }
        Err(e) => warn!("Got a connection error as expected: {e:?}"),
    }

    let disconnected = TetherAgentOptionsBuilder::new("tester")
        .host("tether-io.dev")
        .auto_connect(false)
        .build()
        .expect("this ought initialise but not conect");

    let output = PlugOptionsBuilder::create_output("values")
        .build(&disconnected)
        .expect("this output should be valid always");

    let an_array = &vec![0, 1, 2, 3];
    match disconnected.encode_and_publish(&output, an_array) {
        Ok(()) => panic!("Publish on disconnected agent: This shouldn't work!"),
        Err(e) => warn!("Got a not-connected error as expected: {e:?}"),
    }

    let input_on_disconnected = PlugOptionsBuilder::create_input("something");
    match input_on_disconnected.build(&disconnected) {
        Ok(_) => panic!("Input plug subscribe on disconnected client: This shouldn't work!"),
        Err(e) => warn!("Got a subscribe failure error as expected: {e:?}"),
    }

    // Rust's type-checking kind of prevents this happening at all!
    // let bad_payload: &[u8; 9] = &[0x87, 0xA3, 0x69, 0x6E, 0x74, 0x01, 0xA5, 0x66, 0x6C];
    // match working_tether_agent.encode_and_publish::<CustomStruct>(&output, bad_payload) {
    //     Ok(()) => panic!("Encoding: This shouldn't work!"),
    //     Err(e) => warn!("Got an encoding error as expected: {e:?}"),
    // }

    let working_tether_agent = TetherAgentOptionsBuilder::new("tester")
        .build()
        .expect("this should connect to local server");

    let bad_payload: &[u8; 9] = &[0x87, 0xA3, 0x69, 0x6E, 0x74, 0x01, 0xA5, 0x66, 0x6C];
    working_tether_agent
        .publish(&output, Some(bad_payload))
        .expect("This will produce an error when DECODING, but not checked by library");

    let bad_topic_input =
        PlugOptionsBuilder::create_input("something").topic(Some("*/#/house+".into()));
    match bad_topic_input.build(&working_tether_agent) {
        Ok(_) => panic!("Weird topic: This shouldn't work!"),
        Err(e) => warn!("Got a subscribe error (bad topic) as expected: {e:?}"),
    }
}
