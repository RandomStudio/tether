use anyhow::anyhow;
use log::*;
use rmp_serde::from_slice;
use serde::Deserialize;

use crate::TetherAgent;

use super::{
    definitions::{ChannelDef, ChannelReceiverDef},
    tether_compliant_topic::TetherOrCustomTopic,
};

pub struct ChannelReceiver<'a, T: Deserialize<'a>> {
    definition: ChannelReceiverDef,
    marker: std::marker::PhantomData<&'a T>,
}

impl<'a, T: Deserialize<'a>> ChannelReceiver<'a, T> {
    pub fn new(
        tether_agent: &'a TetherAgent,
        definition: ChannelReceiverDef,
    ) -> anyhow::Result<ChannelReceiver<'a, T>> {
        let topic_string = definition.topic().full_topic_string();

        let channel = ChannelReceiver {
            definition: definition.clone(),
            marker: std::marker::PhantomData,
        };

        // This is really only useful for testing purposes.
        if !tether_agent.auto_connect_enabled() {
            warn!("Auto-connect is disabled, skipping subscription");
            return Ok(channel);
        }

        if let Some(client) = &tether_agent.client {
            match client.subscribe(&topic_string, definition.qos()) {
                Ok(res) => {
                    debug!("This topic was fine: \"{}\"", &topic_string);
                    debug!("Server respond OK for subscribe: {res:?}");
                    Ok(channel)
                }
                Err(_e) => Err(anyhow!("ClientError")),
            }
        } else {
            Err(anyhow!("Client not available for subscription"))
        }
    }

    pub fn definition(&self) -> &ChannelReceiverDef {
        &self.definition
    }

    /// Typically, you do not need to call this function yourself - rather use `.parse` which will
    /// both check if the message belongs this channel AND, if so, decode it as well.
    ///
    /// Uses the topic of an incoming message to check against the definition of an Channel Receiver.
    ///
    /// Due to the use of wildcard subscriptions, multiple topic strings might match a given
    /// Channel Receiver definition. e.g. `someRole/channelMessages` and `anotherRole/channelMessages` and `someRole/channelMessages/specificID`
    /// should ALL match on a Channel Receiver named `channelMessages` unless more specific Role and/or ID
    /// parts were specified in the Channel Receiver Definition.
    ///
    /// In the case where a Channel Receiver was defined with a completely manually-specified topic string,
    /// this function returns a warning and marks ANY incoming message as a valid match; the end-user
    /// developer is expected to match against topic strings themselves.
    pub fn matches(&self, incoming_topic: &TetherOrCustomTopic) -> bool {
        match incoming_topic {
            TetherOrCustomTopic::Tether(incoming_three_parts) => match &self.definition().topic() {
                TetherOrCustomTopic::Tether(my_tpt) => {
                    let matches_role =
                        my_tpt.role() == "+" || my_tpt.role().eq(incoming_three_parts.role());
                    let matches_channel_name = my_tpt.channel_name() == "+"
                        || my_tpt
                            .channel_name()
                            .eq(incoming_three_parts.channel_name());
                    let matches_id = match my_tpt.id() {
                        Some(specified_id) => match incoming_three_parts.id() {
                            Some(incoming_id) => specified_id == incoming_id,
                            None => false,
                        },
                        None => true,
                    };

                    debug!("Test match for Channel named \"{}\" with def {:?} against {:?} => matches_role? {}, matches_id? {}, matches_channel_name? {}",
                        &self.definition().name(), &self.definition().topic(),
                        &incoming_three_parts, matches_role, matches_id, matches_channel_name);
                    matches_role && matches_id && matches_channel_name
                }
                TetherOrCustomTopic::Custom(my_custom_topic) => {
                    debug!(
                    "Custom/manual topic \"{}\" cannot be matched automatically; please filter manually for this",
                    &my_custom_topic,
                );
                    my_custom_topic.as_str() == "#"
                        || my_custom_topic.as_str() == incoming_three_parts.topic()
                }
            },
            TetherOrCustomTopic::Custom(incoming_custom) => match &self.definition().topic {
                TetherOrCustomTopic::Custom(my_custom_topic) => {
                    if my_custom_topic.as_str() == "#"
                        || my_custom_topic.as_str() == incoming_custom.as_str()
                    {
                        true
                    } else {
                        warn!(
                            "Incoming topic \"{}\" is not a Tether-Compliant topic",
                            &incoming_custom
                        );
                        false
                    }
                }
                TetherOrCustomTopic::Tether(_) => {
                    error!("Incoming is NOT Tether Compliant Topic but this Channel DOES have Tether Compliant Topic; cannot decide match");
                    false
                }
            },
        }
    }

    pub fn parse(&self, incoming_topic: &TetherOrCustomTopic, payload: &'a [u8]) -> Option<T> {
        if self.matches(incoming_topic) {
            match from_slice::<T>(payload) {
                Ok(msg) => Some(msg),
                Err(e) => {
                    error!(
                        "Failed to parse message on channel \"{}\": {}",
                        &self.definition().name,
                        e
                    );
                    None
                }
            }
        } else {
            None
        }
    }
}
