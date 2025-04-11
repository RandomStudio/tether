use anyhow::anyhow;
use log::*;
use serde::Deserialize;

use crate::TetherAgent;

use super::{tether_compliant_topic::TetherOrCustomTopic, ChannelCommon};

pub struct ChannelReceiver<'a, T: Deserialize<'a>> {
    name: String,
    topic: TetherOrCustomTopic,
    qos: i32,
    tether_agent: &'a TetherAgent,
    marker: std::marker::PhantomData<T>,
}

impl<'a, T: Deserialize<'a>> ChannelCommon<'a> for ChannelReceiver<'a, T> {
    fn name(&self) -> &str {
        &self.name
    }

    fn generated_topic(&self) -> &str {
        match &self.topic {
            TetherOrCustomTopic::Custom(s) => {
                debug!(
                    "Channel named \"{}\" has custom topic \"{}\"",
                    &self.name, &s
                );
                s
            }
            TetherOrCustomTopic::Tether(t) => {
                debug!(
                    "Channel named \"{}\" has Tether-compliant topic \"{:?}\"",
                    &self.name, t
                );
                t.topic()
            }
        }
    }

    fn topic(&'_ self) -> &'_ TetherOrCustomTopic {
        &self.topic
    }

    fn qos(&self) -> i32 {
        self.qos
    }
}

impl<'a, T: Deserialize<'a>> ChannelReceiver<'a, T> {
    pub fn new(
        tether_agent: &'a TetherAgent,
        name: &str,
        topic: TetherOrCustomTopic,
        qos: Option<i32>,
    ) -> anyhow::Result<ChannelReceiver<'a, T>> {
        let topic_string = topic.full_topic_string();

        let channel = ChannelReceiver {
            name: String::from(name),
            topic,
            qos: qos.unwrap_or(1),
            tether_agent,
            marker: std::marker::PhantomData,
        };

        // This is really only useful for testing purposes.
        if !tether_agent.auto_connect_enabled() {
            warn!("Auto-connect is disabled, skipping subscription");
            return Ok(channel);
        }

        if let Some(client) = &tether_agent.client {
            match client.subscribe(&topic_string, {
                match qos {
                    Some(0) => rumqttc::QoS::AtMostOnce,
                    Some(1) => rumqttc::QoS::AtLeastOnce,
                    Some(2) => rumqttc::QoS::ExactlyOnce,
                    _ => rumqttc::QoS::AtLeastOnce,
                }
            }) {
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

    // /// Use the topic of an incoming message to check against the definition of an Channel Receiver.
    // ///
    // /// Due to the use of wildcard subscriptions, multiple topic strings might match a given
    // /// Channel Receiver definition. e.g. `someRole/channelMessages` and `anotherRole/channelMessages` and `someRole/channelMessages/specificID`
    // /// should ALL match on a Channel Receiver named `channelMessages` unless more specific Role and/or ID
    // /// parts were specified in the Channel Receiver Definition.
    // ///
    // /// In the case where a Channel Receiver was defined with a completely manually-specified topic string,
    // /// this function returns a warning and marks ANY incoming message as a valid match; the end-user
    // /// developer is expected to match against topic strings themselves.
    // pub fn matches(&self, incoming_topic: &TetherOrCustomTopic) -> bool {
    //     match incoming_topic {
    //         TetherOrCustomTopic::Tether(incoming_three_parts) => match &self.topic {
    //             TetherOrCustomTopic::Tether(my_tpt) => {
    //                 let matches_role =
    //                     my_tpt.role() == "+" || my_tpt.role().eq(incoming_three_parts.role());
    //                 let matches_channel_name = my_tpt.channel_name() == "+"
    //                     || my_tpt
    //                         .channel_name()
    //                         .eq(incoming_three_parts.channel_name());
    //                 let matches_id = match my_tpt.id() {
    //                     Some(specified_id) => match incoming_three_parts.id() {
    //                         Some(incoming_id) => specified_id == incoming_id,
    //                         None => false,
    //                     },
    //                     None => true,
    //                 };

    //                 debug!("Test match for Channel named \"{}\" with def {:?} against {:?} => matches_role? {}, matches_id? {}, matches_channel_name? {}", &self.name, &self.topic, &incoming_three_parts, matches_role, matches_id, matches_channel_name);
    //                 matches_role && matches_id && matches_channel_name
    //             }
    //             TetherOrCustomTopic::Custom(my_custom_topic) => {
    //                 debug!(
    //                 "Custom/manual topic \"{}\" on Channel \"{}\" cannot be matched automatically; please filter manually for this",
    //                 &my_custom_topic,
    //                 self.name()
    //             );
    //                 my_custom_topic.as_str() == "#"
    //                     || my_custom_topic.as_str() == incoming_three_parts.topic()
    //             }
    //         },
    //         TetherOrCustomTopic::Custom(incoming_custom) => match &self.topic {
    //             TetherOrCustomTopic::Custom(my_custom_topic) => {
    //                 if my_custom_topic.as_str() == "#"
    //                     || my_custom_topic.as_str() == incoming_custom.as_str()
    //                 {
    //                     true
    //                 } else {
    //                     warn!(
    //                         "Incoming topic \"{}\" is not a Tether-Compliant topic",
    //                         &incoming_custom
    //                     );
    //                     false
    //                 }
    //             }
    //             TetherOrCustomTopic::Tether(_) => {
    //                 error!("Incoming is NOT Tether Compliant Topic but this Channel DOES have Tether Compliant Topic; cannot decide match");
    //                 false
    //             }
    //         },
    //     }
    // }
    //
    //
    //
}
