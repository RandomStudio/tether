use std::{fs::File, io::BufReader};

use clap::Args;
use log::{debug, error, info, warn};
use rmp_serde::from_slice;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tether_agent::{
    OutputPlugDefinition, PlugDefinition, PlugDefinitionCommon, PlugOptionsBuilder, TetherAgent,
    TetherAgentOptionsBuilder,
};

use crate::{defaults, Cli};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct SimulationMessage {
    r#type: String,
    data: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct SimulationRow {
    topic: String,
    message: SimulationMessage,
    delta_time: u64,
}

#[derive(Args)]
pub struct PlaybackOptions {
    /// Overide any original topics saved in the file, to use with every published message
    #[arg(long = "file.path", default_value_t=String::from("./demo.json"))]
    file_path: String,

    /// Overide any original topics saved in the file, to use with every published message
    #[arg(long = "overrideTopic")]
    override_topic: Option<String>,

    /// Flag to enable infinite looping for playback
    #[arg(long = "loops.infinite")]
    loop_infinite: bool,
}

pub fn playback(cli: &Cli, options: &PlaybackOptions) {
    info!("Tether Playback Utility");

    let tether_agent = TetherAgentOptionsBuilder::new(defaults::AGENT_ROLE)
        .host(&cli.tether_host)
        .port(cli.tether_port)
        .username(&cli.tether_username)
        .password(&cli.tether_password)
        .build()
        .expect("failed to connect Tether");

    // let output = PlugOptionsBuilder::create_output(&options.plug_name)
    //     .topic(&publish_topic)
    //     .build(&tether_agent);

    if options.loop_infinite {
        info!("Infinite loops requested; Press Ctr+C to stop");
        loop {
            parse_json_rows(&options.file_path, &tether_agent);
        }
    } else {
        parse_json_rows(&options.file_path, &tether_agent);
    }
}

fn parse_json_rows(filename: &str, tether_agent: &TetherAgent) {
    let file = File::open(filename).expect(&format!("failed to open file {}", filename));
    let reader = BufReader::new(file);
    let deserializer = serde_json::Deserializer::from_reader(reader);
    // let mut rows: Vec<T> = vec![];
    let mut iterator = deserializer.into_iter::<Vec<Value>>();
    let top_level_array: Vec<Value> = iterator.next().unwrap().unwrap();
    for row_value in top_level_array.into_iter() {
        let row: SimulationRow =
            serde_json::from_value(row_value).expect("failed to decode JSON row");

        let SimulationRow {
            topic,
            message,
            delta_time,
        } = &row;

        let payload = &message.data;

        debug!("Sleeping {}ms ...", delta_time);
        std::thread::sleep(std::time::Duration::from_millis(*delta_time));

        tether_agent
            .publish_raw(&topic, payload, None, None)
            .expect("failed to publish");

        debug!("Got row {:?}", row);
    }
}
