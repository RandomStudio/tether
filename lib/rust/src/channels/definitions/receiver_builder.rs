use crate::tether_compliant_topic::{TetherCompliantTopic, TetherOrCustomTopic};

use log::*;

use super::{ChannelBuilder, ChannelReceiverDefinition};

pub struct ChannelReceiverBuilder {
    channel_name: String,
    qos: Option<i32>,
    override_subscribe_role: Option<String>,
    override_subscribe_id: Option<String>,
    override_subscribe_channel_name: Option<String>,
    override_topic: Option<String>,
}

impl ChannelBuilder for ChannelReceiverBuilder {
    fn new(name: &str) -> Self {
        ChannelReceiverBuilder {
            channel_name: String::from(name),
            override_subscribe_id: None,
            override_subscribe_role: None,
            override_subscribe_channel_name: None,
            override_topic: None,
            qos: None,
        }
    }

    fn qos(self, qos: Option<i32>) -> Self {
        ChannelReceiverBuilder { qos, ..self }
    }

    fn role(self, role: Option<&str>) -> Self {
        if self.override_topic.is_some() {
            error!("Override topic was also provided; this will take precedence");
            self
        } else {
            let override_subscribe_role = role.map(|s| s.into());
            ChannelReceiverBuilder {
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
            ChannelReceiverBuilder {
                override_subscribe_id,
                ..self
            }
        }
    }

    fn override_name(self, override_channel_name: Option<&str>) -> Self {
        if self.override_topic.is_some() {
            error!("Override topic was also provided; this will take precedence");
            return self;
        }
        if override_channel_name.is_some() {
            let override_subscribe_channel_name = override_channel_name.map(|s| s.into());
            ChannelReceiverBuilder {
                override_subscribe_channel_name,
                ..self
            }
        } else {
            debug!("Override Channel name set to None; will use original name \"{}\" given in ::create_receiver constructor", self.channel_name);
            self
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
                ChannelReceiverBuilder {
                    override_topic: Some(t.into()),
                    ..self
                }
            }
            None => ChannelReceiverBuilder {
                override_topic: None,
                ..self
            },
        }
    }
}

impl ChannelReceiverBuilder {
    pub fn any_channel(self) -> Self {
        ChannelReceiverBuilder {
            override_subscribe_channel_name: Some("+".into()),
            ..self
        }
    }

    pub fn build(self) -> ChannelReceiverDefinition {
        let tpt: TetherOrCustomTopic = match self.override_topic {
            Some(custom) => TetherOrCustomTopic::Custom(custom),
            None => {
                debug!(
                    "Not a custom topic; provided overrides: role = {:?}, id = {:?}, name = {:?}",
                    self.override_subscribe_role,
                    self.override_subscribe_id,
                    self.override_subscribe_channel_name
                );

                TetherOrCustomTopic::Tether(TetherCompliantTopic::new_for_subscribe(
                    &self
                        .override_subscribe_channel_name
                        .unwrap_or(self.channel_name.clone()),
                    self.override_subscribe_role.as_deref(),
                    self.override_subscribe_id.as_deref(),
                ))
            }
        };

        ChannelReceiverDefinition {
            name: self.channel_name,
            generated_topic: tpt.full_topic_string(),
            topic: tpt,
            qos: self.qos.unwrap_or(1),
        }
    }
}
