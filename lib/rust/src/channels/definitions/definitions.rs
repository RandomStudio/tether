use crate::tether_compliant_topic::TetherOrCustomTopic;

pub trait ChannelDefinition<'a> {
    fn name(&'a self) -> &'a str;
    /// Return the generated topic string actually used by the Channel
    fn generated_topic(&'a self) -> &'a str;
    /// Return the custom or Tether-compliant topic
    fn topic(&'a self) -> &'a TetherOrCustomTopic;
    fn qos(&'a self) -> i32;
}

#[derive(Clone)]
pub struct ChannelSenderDefinition {
    pub name: String,
    pub generated_topic: String,
    pub topic: TetherOrCustomTopic,
    pub qos: i32,
    pub retain: bool,
}

impl ChannelSenderDefinition {
    pub fn retain(&self) -> bool {
        self.retain
    }
}

#[derive(Clone)]
pub struct ChannelReceiverDefinition {
    pub name: String,
    pub generated_topic: String,
    pub topic: TetherOrCustomTopic,
    pub qos: i32,
}

impl<'a> ChannelDefinition<'a> for ChannelSenderDefinition {
    fn name(&'a self) -> &'a str {
        &self.name
    }

    fn generated_topic(&'a self) -> &'a str {
        &self.generated_topic
    }

    fn topic(&'a self) -> &'a TetherOrCustomTopic {
        &self.topic
    }

    fn qos(&'a self) -> i32 {
        self.qos
    }
}

impl<'a> ChannelDefinition<'a> for ChannelReceiverDefinition {
    fn name(&'a self) -> &'a str {
        &self.name
    }

    fn generated_topic(&'a self) -> &'a str {
        &self.generated_topic
    }

    fn topic(&'a self) -> &'a TetherOrCustomTopic {
        &self.topic
    }

    fn qos(&'a self) -> i32 {
        self.qos
    }
}
