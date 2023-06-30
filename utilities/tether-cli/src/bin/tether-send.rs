use serde::Serialize;
use tether_agent::{PlugOptionsBuilder, TetherAgentOptionsBuilder};

#[derive(Serialize)]
struct DummyData {
    id: usize,
    a_float: f32,
    an_int_array: Vec<usize>,
    a_string: String,
}

fn main() {
    let tether_agent = TetherAgentOptionsBuilder::new("testAgent")
        .build()
        .expect("failed to connect Tether");

    let output = PlugOptionsBuilder::create_output("dummy").build(&tether_agent);

    let dummy_data = DummyData {
        id: 0,
        a_float: 42.0,
        an_int_array: vec![1, 2, 3, 4],
        a_string: "hello world".into(),
    };
    tether_agent
        .encode_and_publish(&output, &dummy_data)
        .expect("failed to publish");
}
