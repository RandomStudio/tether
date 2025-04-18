use crate::{
    tether_compliant_topic::{TetherCompliantTopic, TetherOrCustomTopic},
    TetherAgent,
};

use log::*;

use super::{ChannelDefBuilder, ChannelReceiverDef};

pub struct ChannelReceiverDefBuilder {
    channel_name: String,
    qos: Option<i32>,
    override_subscribe_role: Option<String>,
    override_subscribe_id: Option<String>,
    override_topic: Option<String>,
}

impl ChannelDefBuilder for ChannelReceiverDefBuilder {
    /// Use the ChannelReceiverDefBuilder for creating a **custom** Definition that can
    /// be passed to Tether Agent `.create_receiver_with_defintion`.
    ///
    /// If you don't need a custom Definition, simply use Tether Agent `.create_receiver` instead.
    ///
    /// First call .new(), finalise with .build() to get the ChannelReceiverDefinition.
    fn new(name: &str) -> Self {
        ChannelReceiverDefBuilder {
            channel_name: String::from(name),
            override_subscribe_id: None,
            override_subscribe_role: None,
            override_topic: None,
            qos: None,
        }
    }

    fn qos(self, qos: Option<i32>) -> Self {
        ChannelReceiverDefBuilder { qos, ..self }
    }

    fn role(self, role: Option<&str>) -> Self {
        if self.override_topic.is_some() {
            error!("Override topic was also provided; this will take precedence");
            self
        } else {
            let override_subscribe_role = role.map(|s| s.into());
            ChannelReceiverDefBuilder {
                override_subscribe_role,
                ..self
            }
        }
    }

    fn id(self, id: Option<&str>) -> Self {
        if self.override_topic.is_some() {
            error!("Override topic was also provided; this will take precedence");
            self
        } else {
            let override_subscribe_id = id.map(|s| s.into());
            ChannelReceiverDefBuilder {
                override_subscribe_id,
                ..self
            }
        }
    }

    fn override_name(self, override_channel_name: Option<&str>) -> Self {
        debug!(
            "Override channel name explicity? Might use '{:?}' instead of '{}' ...",
            override_channel_name, self.channel_name
        );
        if self.override_topic.is_some() {
            error!("Override topic was also provided; this will take precedence");
            return self;
        }
        match override_channel_name {
            Some(n) => {
                warn!(
                    "Override channel name explicity, use '{:?}' instead of '{}'",
                    override_channel_name, self.channel_name
                );
                ChannelReceiverDefBuilder {
                    channel_name: n.into(),
                    ..self
                }
            }
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
                ChannelReceiverDefBuilder {
                    override_topic: Some(t.into()),
                    ..self
                }
            }
            None => ChannelReceiverDefBuilder {
                override_topic: None,
                ..self
            },
        }
    }
}

impl ChannelReceiverDefBuilder {
    pub fn any_channel(self) -> Self {
        ChannelReceiverDefBuilder {
            channel_name: "+".into(),
            ..self
        }
    }

    pub fn build(self, tether_agent: &TetherAgent) -> ChannelReceiverDef {
        let tpt: TetherOrCustomTopic = match self.override_topic {
            Some(custom) => TetherOrCustomTopic::Custom(custom),
            None => {
                debug!(
                    "Not a custom topic; provided overrides: role = {:?}, id = {:?}",
                    self.override_subscribe_role, self.override_subscribe_id,
                );

                let optional_id_part = match self.override_subscribe_id {
                    Some(id) => {
                        debug!("Subscribe ID was overriden at Channel options level. The Agent ID will be ignored.");
                        Some(id)
                    }
                    None => {
                        debug!("Subscribe ID was not overriden at Channel options level. The Agent ID will be used instead, if specified in Agent creation.");
                        tether_agent.id().map(String::from)
                    }
                };

                TetherOrCustomTopic::Tether(TetherCompliantTopic::new_for_subscribe(
                    &self.channel_name,
                    self.override_subscribe_role.as_deref(),
                    optional_id_part.as_deref(),
                ))
            }
        };

        ChannelReceiverDef {
            name: self.channel_name,
            generated_topic: tpt.full_topic_string(),
            topic: tpt,
            qos: self.qos.unwrap_or(1),
        }
    }
}
