use tether_agent::{PlugOptionsBuilder, TetherAgentOptionsBuilder};

fn main() {
    let tether_agent = TetherAgentOptionsBuilder::new("example")
        .id("customId")
        .host("localhost")
        .port(1883)
        .username("tether")
        .password("sp_ceB0ss!")
        .build()
        .expect("failed to create Tether Agent");

    let _output_plug = PlugOptionsBuilder::create_output("anOutput")
        .qos(2)
        .retain(true)
        .build(&tether_agent);
    let _input_plug = PlugOptionsBuilder::create_input("everything")
        .topic("#")
        .build(&tether_agent);

    // And then proceed as usual!
}
