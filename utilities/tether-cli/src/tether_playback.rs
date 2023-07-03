use std::{fs::File, io::BufReader};

use clap::Args;
use log::{debug, error, info, warn};
use serde::Serialize;
use serde_json::Value;
use tether_agent::{PlugOptionsBuilder, TetherAgentOptionsBuilder};

use crate::{defaults, Cli};

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

    parse_json_rows(&options.file_path);
}

fn parse_json_rows(filename: &str) {
    let file = File::open(filename).expect(&format!("failed to open file {}", filename));
    let reader = BufReader::new(file);
    let deserializer = serde_json::Deserializer::from_reader(reader);
    // let mut rows: Vec<T> = vec![];
    let mut iterator = deserializer.into_iter::<Vec<Value>>();
    let top_level_array: Vec<Value> = iterator.next().unwrap().unwrap();
    for row_value in top_level_array.into_iter() {
        let row: Value = serde_json::from_value(row_value).unwrap();
        debug!("Got row {:?}", row);
    }
}
