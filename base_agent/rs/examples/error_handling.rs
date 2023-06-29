use env_logger::{Builder, Env};
use log::{debug, error, info};
use tether_agent::TetherAgentOptionsBuilder;

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
            error!("This shouldn't work!");
        }
        Err(e) => info!("Got an error as expected: {e:?}"),
    }
}
