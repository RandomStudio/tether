use crate::TetherAgent;
use serde::Serialize;

use super::definitions::ChannelSenderDef;

pub struct ChannelSender<T: Serialize> {
    definition: ChannelSenderDef,
    marker: std::marker::PhantomData<T>,
}

impl<T: Serialize> ChannelSender<T> {
    pub fn new(definition: ChannelSenderDef) -> ChannelSender<T> {
        ChannelSender {
            definition,
            marker: std::marker::PhantomData,
        }
    }

    pub fn definition(&self) -> &ChannelSenderDef {
        &self.definition
    }

    pub fn send(&self, tether_agent: &TetherAgent, payload: &T) -> anyhow::Result<()> {
        tether_agent.send(self, payload)
    }

    pub fn send_raw(&self, tether_agent: &TetherAgent, payload: &[u8]) -> anyhow::Result<()> {
        tether_agent.send_raw(self.definition(), Some(payload))
    }
}
