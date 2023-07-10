use std::{
    fs::File,
    io::BufReader,
    sync::mpsc::{self, Receiver},
};

use clap::Args;
use log::{debug, info, warn};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tether_agent::TetherAgent;

#[derive(Args, Clone)]
pub struct PlaybackOptions {
    /// Specify the full path to the JSON file containing recorded messages
    #[arg(long = "file.path", default_value_t=String::from("./demo.json"))]
    pub file_path: String,

    /// Overide any original topics saved in the file, to use with every published message
    #[arg(long = "topic")]
    pub override_topic: Option<String>,

    /// How many times to loop playback
    #[arg(long = "loops.count", default_value_t = 1)]
    pub loop_count: usize,

    /// Flag to enable infinite looping for playback (ignore loops.count if enabled)
    #[arg(long = "loops.infinite")]
    pub loop_infinite: bool,

    /// Flag to disable registration of Ctrl+C handler - this is usually necessary
    /// when using the utility programmatically (i.e. not via CLI)
    #[arg(long = "ignoreCtrlC")]
    pub ignore_ctrl_c: bool,
}

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

pub struct TetherPlaybackUtil {
    stop_request_tx: mpsc::Sender<bool>,
    stop_request_rx: mpsc::Receiver<bool>,
    options: PlaybackOptions,
    tether_agent: TetherAgent,
}

impl TetherPlaybackUtil {
    pub fn new(options: PlaybackOptions, tether_agent: TetherAgent) -> Self {
        info!("Tether Playback Utility: initialise");

        let (tx, rx) = mpsc::channel();
        TetherPlaybackUtil {
            stop_request_tx: tx,
            stop_request_rx: rx,
            options,
            tether_agent,
        }
    }

    pub fn get_stop_tx(&self) -> mpsc::Sender<bool> {
        self.stop_request_tx.clone()
    }

    pub fn start(&self) {
        info!("Tether Playback Utility: start playback");

        if let Some(t) = &self.options.override_topic {
            warn!("Override topic provided; ALL topics in JSON entries will be ignored and replaced with \"{}\"",t);
        }

        let stop_from_key = self.stop_request_tx.clone();

        if !self.options.ignore_ctrl_c {
            warn!("Infinite loops requested; Press Ctr+C to stop");
            ctrlc::set_handler(move || {
                // should_stop_clone.store(true, Ordering::Relaxed);
                stop_from_key
                    .send(true)
                    .expect("failed to send stop from key");
                warn!("received Ctrl+C! 2");
            })
            .expect("Error setting Ctrl-C handler");
        } else {
            warn!(
                "No Ctrl+C handler set; you may need to kill this process manually, PID: {}",
                std::process::id()
            );
        }

        let mut finished = false;

        let mut count = 0;

        while !finished {
            count += 1;
            if !finished {
                if !self.options.loop_infinite {
                    info!(
                        "Finite loops requested: starting loop {}/{}",
                        count, self.options.loop_count
                    );
                } else {
                    info!("Infinite loops requested; starting loop {}", count);
                }
                if parse_json_rows(
                    &self.options.file_path,
                    &self.tether_agent,
                    &self.options.override_topic,
                    &self.stop_request_rx,
                ) {
                    warn!("Early exit; finish now");
                    finished = true;
                }
            }
            if !self.options.loop_infinite && count >= self.options.loop_count {
                info!("All {} loops completed", &self.options.loop_count);
                finished = true;
            }
        }
    }
}

fn parse_json_rows(
    filename: &str,
    tether_agent: &TetherAgent,
    override_topic: &Option<String>,
    should_stop_rx: &Receiver<bool>,
) -> bool {
    let file = File::open(filename).unwrap_or_else(|_| panic!("failed to open file {}", filename));
    let reader = BufReader::new(file);
    let deserializer = serde_json::Deserializer::from_reader(reader);
    let mut iterator = deserializer.into_iter::<Vec<Value>>();
    let top_level_array: Vec<Value> = iterator.next().unwrap().unwrap();

    let mut finished = false;
    let mut early_exit = false;
    // let rows = top_level_array.into_iter();

    let mut index = 0;

    while !finished {
        while let Ok(_should_stop) = should_stop_rx.try_recv() {
            early_exit = true;
            finished = true;
        }
        if let Some(row_value) = top_level_array.get(index) {
            let row: SimulationRow =
                serde_json::from_value(row_value.clone()).expect("failed to decode JSON row");

            let SimulationRow {
                topic,
                message,
                delta_time,
            } = &row;

            let payload = &message.data;

            if !finished {
                debug!("Sleeping {}ms ...", delta_time);
                std::thread::sleep(std::time::Duration::from_millis(*delta_time));
                let topic = match &override_topic {
                    Some(t) => String::from(t),
                    None => String::from(topic),
                };

                tether_agent
                    .publish_raw(&topic, payload, None, None)
                    .expect("failed to publish");
            }

            debug!("Got row {:?}", row);
        } else {
            finished = true;
        }
        index += 1;
    }
    early_exit
}
