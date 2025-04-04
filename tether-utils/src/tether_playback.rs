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
    /// JSON file to load recording from
    #[arg(default_value_t=String::from("./demo.json"))]
    pub file_path: String,

    /// Comma-separated strings used to filter messages by topic
    #[arg(long = "topic.filter")]
    pub topic_filters: Option<String>,

    /// Overide any original topics saved in the file, to use with every published message
    #[arg(long = "topic.override")]
    pub override_topic: Option<String>,

    /// Speed up or slow down playback (e.g. 2.0 = double speed)
    #[arg(long = "playback.speed", default_value_t = 1.0)]
    pub playback_speed: f32,

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

impl Default for PlaybackOptions {
    fn default() -> Self {
        PlaybackOptions {
            file_path: "./demo.json".into(),
            override_topic: None,
            loop_count: 1,
            loop_infinite: false,
            ignore_ctrl_c: false,
            playback_speed: 1.0,
            topic_filters: None,
        }
    }
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
}

impl TetherPlaybackUtil {
    pub fn new(options: PlaybackOptions) -> Self {
        info!("Tether Playback Utility: initialise");

        let (tx, rx) = mpsc::channel();
        TetherPlaybackUtil {
            stop_request_tx: tx,
            stop_request_rx: rx,
            options,
        }
    }

    pub fn get_stop_tx(&self) -> mpsc::Sender<bool> {
        self.stop_request_tx.clone()
    }

    pub fn start(&self, tether_agent: &TetherAgent) {
        info!("Tether Playback Utility: start playback");

        let filters: Option<Vec<String>> = self
            .options
            .topic_filters
            .as_ref()
            .map(|f_string| f_string.split(',').map(String::from).collect());

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
                    tether_agent,
                    &filters,
                    &self.options.override_topic,
                    &self.stop_request_rx,
                    self.options.playback_speed,
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

/// Parse rows and return true when finished
fn parse_json_rows(
    filename: &str,
    tether_agent: &TetherAgent,
    filters: &Option<Vec<String>>,
    override_topic: &Option<String>,
    should_stop_rx: &Receiver<bool>,
    speed_factor: f32,
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

            let should_send: bool = match filters {
                Some(filters) => filters.iter().map(String::from).any(|f| topic.contains(&f)),
                None => true,
            };

            if should_send {
                let payload = &message.data;

                if !finished {
                    let delta_time = *delta_time as f64 / speed_factor as f64;
                    debug!("Sleeping {}ms ...", delta_time);
                    std::thread::sleep(std::time::Duration::from_millis(delta_time as u64));
                    let topic = match &override_topic {
                        Some(t) => String::from(t),
                        None => String::from(topic),
                    };

                    tether_agent
                        .publish_raw(&topic, payload, None, None)
                        .expect("failed to publish");
                }
            }

            debug!("Got row {:?}", row);
        } else {
            finished = true;
        }
        index += 1;
    }
    early_exit
}
