use crate::{
    tether_compliant_topic::{TetherCompliantTopic, TetherOrCustomTopic},
    TetherAgent,
};

use log::*;

use super::{ChannelDefBuilder, ChannelSenderDef};

pub struct ChannelSenderDefBuilder {
    channel_name: String,
    qos: Option<i32>,
    override_publish_role: Option<String>,
    override_publish_id: Option<String>,
    override_topic: Option<String>,
    retain: Option<bool>,
}

impl ChannelDefBuilder for ChannelSenderDefBuilder {
    /// Use the ChannelSenderDefBuilder for creating a **custom** Definition that can
    /// be passed to Tether Agent `.create_sender_with_defintion`.
    ///
    /// If you don't need a custom Definition, simply use Tether Agent `.create_sender` instead.
    ///
    /// First call .new(), finalise with .build() to get the ChannelSenderDefinition.
    fn new(name: &str) -> Self {
        ChannelSenderDefBuilder {
            channel_name: String::from(name),
            override_publish_id: None,
            override_publish_role: None,
            override_topic: None,
            retain: None,
            qos: None,
        }
    }

    fn qos(self, qos: Option<i32>) -> Self {
        ChannelSenderDefBuilder { qos, ..self }
    }

    fn role(self, role: Option<&str>) -> Self {
        if self.override_topic.is_some() {
            error!("Override topic was also provided; this will take precedence");
            self
        } else {
            let override_publish_role = role.map(|s| s.into());
            ChannelSenderDefBuilder {
                override_publish_role,
                ..self
            }
        }
    }

    fn id(self, id: Option<&str>) -> Self {
        if self.override_topic.is_some() {
            error!("Override topic was also provided; this will take precedence");
            self
        } else {
            let override_publish_id = id.map(|s| s.into());
            ChannelSenderDefBuilder {
                override_publish_id,
                ..self
            }
        }
    }

    fn override_name(self, override_channel_name: Option<&str>) -> Self {
        if self.override_topic.is_some() {
            error!("Override topic was also provided; this will take precedence");
            return self;
        }
        match override_channel_name {
            Some(n) => ChannelSenderDefBuilder {
                channel_name: n.into(),
                ..self
            },
            None => {
                debug!("Override Channel name set to None; will use original name \"{}\" given in ::create_receiver constructor", self.channel_name);
                self
            }
        }
    }

    fn override_topic(self, override_topic: Option<&str>) -> Self {
        match override_topic {
            Some(t) => {
                if TryInto::<TetherCompliantTopic>::try_into(t).is_ok() {
                    info!("Custom topic passes Tether Compliant Topic validation");
                } else if t == "#" {
                    info!("Wildcard \"#\" custom topics are not Tether Compliant Topics but are valid");
                } else {
                    warn!(
                        "Could not convert \"{}\" into Tether Compliant Topic; presumably you know what you're doing!",
                        t
                    );
                }
                ChannelSenderDefBuilder {
                    override_topic: Some(t.into()),
                    ..self
                }
            }
            None => ChannelSenderDefBuilder {
                override_topic: None,
                ..self
            },
        }
    }
}

impl ChannelSenderDefBuilder {
    pub fn retain(self, should_retain: Option<bool>) -> Self {
        ChannelSenderDefBuilder {
            retain: should_retain,
            ..self
        }
    }

    pub fn build(self, tether_agent: &TetherAgent) -> ChannelSenderDef {
        let tpt: TetherOrCustomTopic = match self.override_topic {
            Some(custom) => {
                warn!(
                    "Custom topic override: \"{}\" - all other options ignored",
                    custom
                );
                TetherOrCustomTopic::Custom(custom)
            }
            None => {
                let optional_id_part = match self.override_publish_id {
                    Some(id) => {
                        debug!("Publish ID was overriden at Channel options level. The Agent ID will be ignored.");
                        Some(id)
                    }
                    None => {
                        debug!("Publish ID was not overriden at Channel options level. The Agent ID will be used instead, if specified in Agent creation.");
                        tether_agent.id().map(String::from)
                    }
                };

                TetherOrCustomTopic::Tether(TetherCompliantTopic::new_for_publish(
                    tether_agent,
                    &self.channel_name,
                    self.override_publish_role.as_deref(),
                    optional_id_part.as_deref(),
                ))
            }
        };

        ChannelSenderDef {
            name: self.channel_name,
            generated_topic: tpt.full_topic_string(),
            topic: tpt,
            qos: self.qos.unwrap_or(1),
            retain: self.retain.unwrap_or(false),
        }
    }
}
