use tether_agent::{PlugOptionsBuilder, TetherAgentOptionsBuilder};

fn main() {
    let tether_agent = TetherAgentOptionsBuilder::new("example")
        .id_optional(None)
        .host("localhost")
        .port(1883)
        .username("tether")
        .password("sp_ceB0ss!")
        .build()
        .expect("failed to create Tether Agent");

    let output_plug = PlugOptionsBuilder::create_output("anOutput")
        .qos(2)
        .retain(true)
        .build(&tether_agent);
    let input_plug = PlugOptionsBuilder::create_input("everything")
        .topic("#")
        .build(&tether_agent);

    println!("Agent looks like this: {:?}", tether_agent.description());
    let (role, id, _) = tether_agent.description();
    assert_eq!(role, "example");
    assert_eq!(id, "any"); // because we set None

    println!("output plug: {:?}", output_plug);
    println!("input plug: {:?}", input_plug);
}
