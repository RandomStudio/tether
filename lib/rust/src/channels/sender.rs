use crate::TetherAgent;
use serde::Serialize;

use super::definitions::ChannelSenderDefinition;

pub struct ChannelSender<'a, T: Serialize> {
    definition: ChannelSenderDefinition,
    tether_agent: &'a TetherAgent,
    marker: std::marker::PhantomData<T>,
}

impl<'a, T: Serialize> ChannelSender<'a, T> {
    pub fn new(
        tether_agent: &'a TetherAgent,
        definition: ChannelSenderDefinition,
    ) -> ChannelSender<'a, T> {
        ChannelSender {
            definition,
            tether_agent,
            marker: std::marker::PhantomData,
        }
    }

    pub fn definition(&self) -> &ChannelSenderDefinition {
        &self.definition
    }

    pub fn send(&self, payload: &T) -> anyhow::Result<()> {
        self.tether_agent.send(self, payload)
    }

    pub fn send_raw(&self, payload: &[u8]) -> anyhow::Result<()> {
        self.tether_agent.send_raw(self.definition(), Some(payload))
    }
}
