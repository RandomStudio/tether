use crate::TetherAgent;
use anyhow::anyhow;
use log::*;
use rmp_serde::to_vec_named;
use serde::Serialize;

use super::{tether_compliant_topic::TetherOrCustomTopic, ChannelCommon};

pub struct ChannelSender<'a> {
    name: String,
    topic: TetherOrCustomTopic,
    qos: i32,
    retain: bool,
    tether_agent: &'a TetherAgent,
}

impl<'a> ChannelCommon<'a> for ChannelSender<'a> {
    fn name(&'_ self) -> &'_ str {
        &self.name
    }

    fn generated_topic(&self) -> &str {
        match &self.topic {
            TetherOrCustomTopic::Custom(s) => s,
            TetherOrCustomTopic::Tether(t) => t.topic(),
        }
    }

    fn topic(&'_ self) -> &'_ TetherOrCustomTopic {
        &self.topic
    }

    fn qos(&'_ self) -> i32 {
        self.qos
    }
}

impl<'a> ChannelSender<'a> {
    pub fn new(
        name: &str,
        topic: TetherOrCustomTopic,
        qos: Option<i32>,
        retain: Option<bool>,
        tether_agent: &'a TetherAgent,
    ) -> ChannelSender<'a> {
        ChannelSender {
            name: String::from(name),
            topic,
            qos: qos.unwrap_or(1),
            retain: retain.unwrap_or(false),
            tether_agent,
        }
    }

    pub fn retain(&self) -> bool {
        self.retain
    }

    pub fn send_raw(&self, payload: &[u8]) -> anyhow::Result<()> {
        if let Some(client) = &self.tether_agent.client {
            let res = client
                .publish(
                    self.generated_topic(),
                    rumqttc::QoS::AtLeastOnce,
                    false,
                    payload,
                )
                .map_err(anyhow::Error::msg);
            res
        } else {
            Err(anyhow!("no client"))
        }
    }

    pub fn send<T: Serialize>(&self, payload: T) -> anyhow::Result<()> {
        match to_vec_named(&payload) {
            Ok(data) => self.send_raw(&data),
            Err(e) => Err(e.into()),
        }
    }
}
