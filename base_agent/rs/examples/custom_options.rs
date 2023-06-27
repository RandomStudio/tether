use tether_agent::TetherAgentOptionsBuilder;

fn main() {
    let agent = TetherAgentOptionsBuilder::new("example")
        .id("customId")
        .finalize().expect("failed to create Tether Agent");

    let output_plug = agent.create_output_plug(name, qos, retain, override_topic)
}
