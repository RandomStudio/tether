use crate::TetherAgent;
use anyhow::anyhow;
use rmp_serde::to_vec_named;
use serde::Serialize;

use super::{
    options::definitions::{ChannelDefinition, ChannelSenderDefinition},
    tether_compliant_topic::TetherOrCustomTopic,
};

pub struct ChannelSender<'a, T: Serialize> {
    name: String,
    topic: TetherOrCustomTopic,
    qos: i32,
    retain: bool,
    tether_agent: &'a TetherAgent,
    marker: std::marker::PhantomData<T>,
}

impl<'a, T: Serialize> ChannelDefinition<'a> for ChannelSender<'a, T> {
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

impl<'a, T: Serialize> ChannelSender<'a, T> {
    pub fn new(
        tether_agent: &'a TetherAgent,
        definition: ChannelSenderDefinition,
    ) -> ChannelSender<'a, T> {
        ChannelSender {
            name: String::from(definition.name()),
            topic: definition.topic().clone(),
            qos: definition.qos(),
            retain: definition.retain(),
            tether_agent,
            marker: std::marker::PhantomData,
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

    pub fn send(&self, payload: T) -> anyhow::Result<()> {
        match to_vec_named(&payload) {
            Ok(data) => self.send_raw(&data),
            Err(e) => Err(e.into()),
        }
    }
}
