use std::{fs::File, io::BufReader};

use clap::Args;
use log::{debug, info, warn};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tether_agent::{TetherAgent, TetherAgentOptionsBuilder};

use crate::{defaults, Cli};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SimulationMessage {
    pub r#type: String,
    pub data: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SimulationRow {
    pub topic: String,
    pub message: SimulationMessage,
    pub delta_time: u64,
}

#[derive(Args)]
pub struct PlaybackOptions {
    /// Specify the full path to the JSON file containing recorded messages
    #[arg(long = "file.path", default_value_t=String::from("./demo.json"))]
    file_path: String,

    /// Overide any original topics saved in the file, to use with every published message
    #[arg(long = "topic")]
    override_topic: Option<String>,

    /// How many times to loop playback
    #[arg(long = "loops.count", default_value_t = 1)]
    loop_count: usize,

    /// Flag to enable infinite looping for playback (ignore loops.count if enabled)
    #[arg(long = "loops.infinite")]
    loop_infinite: bool,
}

pub fn playback(cli: &Cli, options: &PlaybackOptions) {
    info!("Tether Playback Utility");

    if let Some(t) = &options.override_topic {
        warn!("Override topic provided; ALL topics in JSON entries will be ignored and replaced with \"{}\"",t);
    }

    let tether_agent = TetherAgentOptionsBuilder::new(defaults::AGENT_ROLE)
        .host(&cli.tether_host)
        .port(cli.tether_port)
        .username(&cli.tether_username)
        .password(&cli.tether_password)
        .build()
        .expect("failed to connect Tether");

    if options.loop_infinite {
        loop {
            warn!("Infinite loops requested; Press Ctr+C to stop");
            parse_json_rows(&options.file_path, &tether_agent, &options.override_topic);
        }
    } else {
        let mut count = 0;
        while count < options.loop_count {
            count += 1;
            info!(
                "Finite loops requested: starting loop {}/{}",
                count, options.loop_count
            );
            parse_json_rows(&options.file_path, &tether_agent, &options.override_topic);
        }
        info!("All {} loops completed", options.loop_count);
    }
}

fn parse_json_rows(filename: &str, tether_agent: &TetherAgent, override_topic: &Option<String>) {
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

        let topic = match &override_topic {
            Some(t) => String::from(t),
            None => String::from(topic),
        };

        tether_agent
            .publish_raw(&topic, payload, None, None)
            .expect("failed to publish");

        debug!("Got row {:?}", row);
    }
}
