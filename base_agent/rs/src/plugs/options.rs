use anyhow::anyhow;
use log::{debug, error, info, warn};

use crate::{
    definitions::{InputPlugDefinition, OutputPlugDefinition, PlugDefinitionCommon},
    three_part_topic::ThreePartTopic,
    PlugDefinition, TetherAgent,
};

use super::three_part_topic::TetherOrCustomTopic;

pub struct InputPlugOptions {
    plug_name: String,
    qos: Option<i32>,
    override_subscribe_role: Option<String>,
    override_subscribe_id: Option<String>,
    override_subscribe_plug_name: Option<String>,
    override_topic: Option<String>,
}

pub struct OutputPlugOptions {
    plug_name: String,
    qos: Option<i32>,
    override_publish_role: Option<String>,
    override_publish_id: Option<String>,
    override_topic: Option<String>,
    retain: Option<bool>,
}

/// This is the definition of an Input or Output Plug.
///
/// You typically don't use an instance of this directly; call `.build()` at the
/// end of the chain to get a usable **PlugDefinition**
pub enum PlugOptionsBuilder {
    InputPlugOptions(InputPlugOptions),
    OutputPlugOptions(OutputPlugOptions),
}

impl PlugOptionsBuilder {
    pub fn create_input(name: &str) -> PlugOptionsBuilder {
        PlugOptionsBuilder::InputPlugOptions(InputPlugOptions {
            plug_name: String::from(name),
            override_subscribe_id: None,
            override_subscribe_role: None,
            override_subscribe_plug_name: None,
            override_topic: None,
            qos: None,
        })
    }

    pub fn create_output(name: &str) -> PlugOptionsBuilder {
        PlugOptionsBuilder::OutputPlugOptions(OutputPlugOptions {
            plug_name: String::from(name),
            override_publish_id: None,
            override_publish_role: None,
            override_topic: None,
            qos: None,
            retain: None,
        })
    }

    pub fn qos(mut self, qos: Option<i32>) -> Self {
        match &mut self {
            PlugOptionsBuilder::InputPlugOptions(s) => s.qos = qos,
            PlugOptionsBuilder::OutputPlugOptions(s) => s.qos = qos,
        };
        self
    }

    /**
    Override the "role" part of the topic that gets generated for this Plug.
    - For Input Plugs, this means you want to be specific about the Role part
      of the topic, instead of using the default wildcard `+` at this location
    - For Output Plugs, this means you want to override the Role part instead
      of using your Agent's "own" Role with which you created the Tether Agent

    If you override the entire topic using `.topic` this will be ignored.
    */
    pub fn role(mut self, role: Option<&str>) -> Self {
        match &mut self {
            PlugOptionsBuilder::InputPlugOptions(s) => {
                if s.override_topic.is_some() {
                    error!("Override topic was also provided; this will take precedence");
                } else {
                    s.override_subscribe_role = role.map(|s| s.into());
                }
            }
            PlugOptionsBuilder::OutputPlugOptions(s) => {
                if s.override_topic.is_some() {
                    error!("Override topic was also provided; this will take precedence");
                } else {
                    s.override_publish_role = role.map(|s| s.into());
                }
            }
        };
        self
    }

    /**
    Override the "id" part of the topic that gets generated for this Plug.
    - For Input Plugs, this means you want to be specific about the ID part
      of the topic, instead of using the default wildcard `+` at this location
    - For Output Plugs, this means you want to override the ID part instead
      of using your Agent's "own" ID which you specified (or left blank, i.e. "any")
      when creating the Tether Agent

    If you override the entire topic using `.topic` this will be ignored.
    */
    pub fn id(mut self, id: Option<&str>) -> Self {
        match &mut self {
            PlugOptionsBuilder::InputPlugOptions(s) => {
                if s.override_topic.is_some() {
                    error!("Override topic was also provided; this will take precedence");
                } else {
                    s.override_subscribe_id = id.map(|s| s.into());
                }
            }
            PlugOptionsBuilder::OutputPlugOptions(s) => {
                if s.override_topic.is_some() {
                    error!("Override topic was also provided; this will take precedence");
                } else {
                    s.override_publish_id = id.map(|s| s.into());
                }
            }
        };
        self
    }

    /// Override the "name" part of the topic that gets generated for this Plug.
    /// This is mainly to facilitate wildcard subscriptions such as
    /// `someRole/someID/+` instead of `someRole/someID/originalPlugName`.
    ///
    /// In the case of Input Topics, a wildcard `+` can be used to substitute
    /// the last part of the topic as in `role/id/+` but will NOT affect the stored "name"
    /// of the Plug Definition itself. Anything else will be ignored with an error.
    ///
    /// Output Plugs will ignore (with an error) any attempt to change the name after
    /// instantiation.
    pub fn name(mut self, override_plug_name: Option<&str>) -> Self {
        match &mut self {
            PlugOptionsBuilder::InputPlugOptions(opt) => {
                if opt.override_topic.is_some() {
                    error!("Override topic was also provided; this will take precedence");
                }
                if let Some(s) = override_plug_name {
                    if s.eq("+") {
                        info!(
                            "Plug Name part given is a wildcard; subscribe topic will use this but (internally) Plug Name will remain \"{}\"", &opt.plug_name
                        );
                    } else {
                        error!("Input Plugs cannot change their name after ::create_input constructor EXCEPT for wildcard \"+\"");
                    }
                    opt.override_subscribe_plug_name = override_plug_name.map(|s| s.into());
                } else {
                    debug!("Override plug name set to None; will use original name \"{}\" given in ::create_input constructor", opt.plug_name);
                }
            }
            PlugOptionsBuilder::OutputPlugOptions(_) => {
                error!(
                    "Output Plugs cannot change their name part after ::create_output constructor"
                );
            }
        };
        self
    }

    /// Call this if you would like your Input plug to match **any plug**.
    /// This is equivalent to `.name(Some("+"))` but is provided for convenience
    /// since it does not require you to remember the wildcard string.
    ///
    /// This also does not prevent you from further restricting the topic
    /// subscription match by Role and/or ID. So, for example, if you are
    /// interested in **all messages** from an Agent with the role `"brain"`,
    /// it is valid to create a plug with `.role("brain").any_plug()` and this
    /// will subscribe to `"brain/+/+"` as expected.
    pub fn any_plug(mut self) -> Self {
        match &mut self {
            PlugOptionsBuilder::InputPlugOptions(opt) => {
                opt.override_subscribe_plug_name = Some("+".into());
            }
            PlugOptionsBuilder::OutputPlugOptions(_) => {
                error!(
                    "Output Plugs cannot change their name part after ::create_output constructor"
                );
            }
        }
        self
    }

    /// Override the final topic to use for publishing or subscribing. The provided topic **will** be checked
    /// against the Tether Three Part Topic convention, but the function **will not** reject topic strings - just
    /// produce a warning. It's therefore valid to use a wildcard such as "#", for Input (subscribing).
    ///
    /// Any customisations specified using `.role(...)` or `.id(...)` will be ignored if this function is called.
    ///
    /// By default, the override_topic is None, but you can specify None explicitly using this function.
    pub fn topic(mut self, override_topic: Option<&str>) -> Self {
        match override_topic {
            Some(t) => {
                if TryInto::<ThreePartTopic>::try_into(t).is_ok() {
                    info!("Custom topic passes Three Part Topic validation");
                } else if t == "#" {
                    info!("Wildcard \"#\" custom topics are not Three Part Topics but are valid");
                } else {
                    warn!(
                        "Could not convert \"{}\" into Tether 3 Part Topic; presumably you know what you're doing!",
                        t
                    );
                }
                match &mut self {
                    PlugOptionsBuilder::InputPlugOptions(s) => s.override_topic = Some(t.into()),
                    PlugOptionsBuilder::OutputPlugOptions(s) => s.override_topic = Some(t.into()),
                };
            }
            None => {
                match &mut self {
                    PlugOptionsBuilder::InputPlugOptions(s) => s.override_topic = None,
                    PlugOptionsBuilder::OutputPlugOptions(s) => s.override_topic = None,
                };
            }
        }
        self
    }

    pub fn retain(mut self, should_retain: Option<bool>) -> Self {
        match &mut self {
            Self::InputPlugOptions(_) => {
                error!("Cannot set retain flag on Input Plug / subscription");
            }
            Self::OutputPlugOptions(s) => {
                s.retain = should_retain;
            }
        }
        self
    }

    /// Finalise the options (substituting suitable defaults if no custom values have been
    /// provided) and return a valid PlugDefinition that you can actually use.
    pub fn build(self, tether_agent: &mut TetherAgent) -> anyhow::Result<PlugDefinition> {
        match self {
            Self::InputPlugOptions(plug_options) => {
                let tpt: TetherOrCustomTopic = match plug_options.override_topic {
                    Some(custom) => TetherOrCustomTopic::Custom(custom),
                    None => {
                        debug!("Not a custom topic; provided overrides: role = {:?}, id = {:?}, name = {:?}", plug_options.override_subscribe_role, plug_options.override_subscribe_id, plug_options.override_subscribe_plug_name);

                        TetherOrCustomTopic::Tether(ThreePartTopic::new_for_subscribe(
                            &plug_options.plug_name,
                            plug_options.override_subscribe_role.as_deref(),
                            plug_options.override_subscribe_id.as_deref(),
                            plug_options.override_subscribe_plug_name.as_deref(),
                        ))
                    }
                };
                let plug_definition =
                    InputPlugDefinition::new(&plug_options.plug_name, tpt, plug_options.qos);
                if let Some(client) = &tether_agent.client {
                    match client.subscribe(
                        plug_definition.topic_str(),
                        match plug_definition.qos() {
                            0 => rumqttc::QoS::AtMostOnce,
                            1 => rumqttc::QoS::AtLeastOnce,
                            2 => rumqttc::QoS::ExactlyOnce,
                            _ => rumqttc::QoS::AtLeastOnce,
                        },
                    ) {
                        Ok(res) => {
                            debug!("This topic was fine: \"{}\"", plug_definition.topic_str());
                            debug!("Server respond OK for subscribe: {res:?}");
                            Ok(PlugDefinition::InputPlug(plug_definition))
                        }
                        Err(_e) => Err(anyhow!("ClientError")),
                    }
                } else {
                    Err(anyhow!("Client not available for subscription"))
                }
            }
            Self::OutputPlugOptions(plug_options) => {
                let tpt: TetherOrCustomTopic = match plug_options.override_topic {
                    Some(custom) => TetherOrCustomTopic::Custom(custom),
                    None => TetherOrCustomTopic::Tether(ThreePartTopic::new_for_publish(
                        plug_options.override_publish_role.as_deref(),
                        plug_options.override_publish_id.as_deref(),
                        &plug_options.plug_name,
                        tether_agent,
                    )),
                };

                let plug_definition = OutputPlugDefinition::new(
                    &plug_options.plug_name,
                    tpt,
                    plug_options.qos,
                    plug_options.retain,
                );
                Ok(PlugDefinition::OutputPlug(plug_definition))
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::{PlugOptionsBuilder, TetherAgentOptionsBuilder};

    // fn verbose_logging() {
    //     use env_logger::{Builder, Env};
    //     let mut logger_builder = Builder::from_env(Env::default().default_filter_or("debug"));
    //     logger_builder.init();
    // }

    #[test]
    fn default_input_plug() {
        // verbose_logging();
        let mut tether_agent = TetherAgentOptionsBuilder::new("tester")
            .build()
            .expect("sorry, these tests require working localhost Broker");
        let input = PlugOptionsBuilder::create_input("one")
            .build(&mut tether_agent)
            .unwrap();
        assert_eq!(input.name(), "one");
        assert_eq!(input.topic(), "+/+/one");
    }

    #[test]
    /// This is a fairly trivial example, but contrast with the test
    /// `output_plug_default_but_agent_id_custom`: although a custom ID was set for the
    /// Agent, this does not affect the Topic for an Input Plug created without any
    /// explicit overrides.
    fn default_input_plug_with_agent_custom_id() {
        // verbose_logging();
        let mut tether_agent = TetherAgentOptionsBuilder::new("tester")
            .id(Some("verySpecialGroup"))
            .build()
            .expect("sorry, these tests require working localhost Broker");
        let input = PlugOptionsBuilder::create_input("one")
            .build(&mut tether_agent)
            .unwrap();
        assert_eq!(input.name(), "one");
        assert_eq!(input.topic(), "+/+/one");
    }

    #[test]
    fn default_output_plug() {
        let mut tether_agent = TetherAgentOptionsBuilder::new("tester")
            .build()
            .expect("sorry, these tests require working localhost Broker");
        let input = PlugOptionsBuilder::create_output("two")
            .build(&mut tether_agent)
            .unwrap();
        assert_eq!(input.name(), "two");
        assert_eq!(input.topic(), "tester/any/two");
    }

    #[test]
    /// This is identical to the case in which an Output Plug is created with defaults (no overrides),
    /// BUT the Agent had a custom ID set, which means that the final topic includes this custom
    /// ID/Group value.
    fn output_plug_default_but_agent_id_custom() {
        let mut tether_agent = TetherAgentOptionsBuilder::new("tester")
            .id(Some("specialCustomGrouping"))
            .build()
            .expect("sorry, these tests require working localhost Broker");
        let input = PlugOptionsBuilder::create_output("somethingStandard")
            .build(&mut tether_agent)
            .unwrap();
        assert_eq!(input.name(), "somethingStandard");
        assert_eq!(
            input.topic(),
            "tester/specialCustomGrouping/somethingStandard"
        );
    }

    #[test]
    fn input_id_andor_role() {
        let mut tether_agent = TetherAgentOptionsBuilder::new("tester")
            .build()
            .expect("sorry, these tests require working localhost Broker");

        let input_role_only = PlugOptionsBuilder::create_input("thePlug")
            .role(Some("specificRole"))
            .build(&mut tether_agent)
            .unwrap();
        assert_eq!(input_role_only.name(), "thePlug");
        assert_eq!(input_role_only.topic(), "specificRole/+/thePlug");

        let input_id_only = PlugOptionsBuilder::create_input("thePlug")
            .id(Some("specificID"))
            .build(&mut tether_agent)
            .unwrap();
        assert_eq!(input_id_only.name(), "thePlug");
        assert_eq!(input_id_only.topic(), "+/specificID/thePlug");

        let input_both = PlugOptionsBuilder::create_input("thePlug")
            .id(Some("specificID"))
            .role(Some("specificRole"))
            .build(&mut tether_agent)
            .unwrap();
        assert_eq!(input_both.name(), "thePlug");
        assert_eq!(input_both.topic(), "specificRole/specificID/thePlug");
    }

    #[test]
    /// If the end-user implicitly specifies the plug name part (does not set it to Some(_)
    /// or None) then the ID and/or Role parts will change but the Plug Name part will
    /// remain the "original" / default
    /// Contrast with input_specific_id_andor_role_no_plugname below.
    fn input_specific_id_andor_role_with_plugname() {
        let mut tether_agent = TetherAgentOptionsBuilder::new("tester")
            .build()
            .expect("sorry, these tests require working localhost Broker");

        let input_role_only = PlugOptionsBuilder::create_input("thePlug")
            .role(Some("specificRole"))
            .build(&mut tether_agent)
            .unwrap();
        assert_eq!(input_role_only.name(), "thePlug");
        assert_eq!(input_role_only.topic(), "specificRole/+/thePlug");

        let input_id_only = PlugOptionsBuilder::create_input("thePlug")
            .id(Some("specificID"))
            .build(&mut tether_agent)
            .unwrap();
        assert_eq!(input_id_only.name(), "thePlug");
        assert_eq!(input_id_only.topic(), "+/specificID/thePlug");

        let input_both = PlugOptionsBuilder::create_input("thePlug")
            .id(Some("specificID"))
            .role(Some("specificRole"))
            .build(&mut tether_agent)
            .unwrap();
        assert_eq!(input_both.name(), "thePlug");
        assert_eq!(input_both.topic(), "specificRole/specificID/thePlug");
    }

    #[test]
    /// Unlike input_specific_id_andor_role_with_plugname, this tests the situation where
    /// the end-user (possibly) specifies the ID and/or Role, but also explicitly
    /// sets the Plug Name to Some("+"), ie. "use a wildcard at this
    /// position instead" - and NOT the original plug name.
    fn input_specific_id_andor_role_no_plugname() {
        let mut tether_agent = TetherAgentOptionsBuilder::new("tester")
            .build()
            .expect("sorry, these tests require working localhost Broker");

        let input_only_plugname_none = PlugOptionsBuilder::create_input("thePlug")
            .name(Some("+"))
            .build(&mut tether_agent)
            .unwrap();
        assert_eq!(input_only_plugname_none.name(), "thePlug");
        assert_eq!(input_only_plugname_none.topic(), "+/+/+");

        let input_role_only = PlugOptionsBuilder::create_input("thePlug")
            .name(Some("+"))
            .role(Some("specificRole"))
            .build(&mut tether_agent)
            .unwrap();
        assert_eq!(input_role_only.name(), "thePlug");
        assert_eq!(input_role_only.topic(), "specificRole/+/+");

        let input_id_only = PlugOptionsBuilder::create_input("thePlug")
            // .name(Some("+"))
            .any_plug() // equivalent to Some("+")
            .id(Some("specificID"))
            .build(&mut tether_agent)
            .unwrap();
        assert_eq!(input_id_only.name(), "thePlug");
        assert_eq!(input_id_only.topic(), "+/specificID/+");

        let input_both = PlugOptionsBuilder::create_input("thePlug")
            .name(Some("+"))
            .id(Some("specificID"))
            .role(Some("specificRole"))
            .build(&mut tether_agent)
            .unwrap();
        assert_eq!(input_both.name(), "thePlug");
        assert_eq!(input_both.topic(), "specificRole/specificID/+");
    }

    #[test]
    fn output_custom() {
        let mut tether_agent = TetherAgentOptionsBuilder::new("tester")
            .build()
            .expect("sorry, these tests require working localhost Broker");

        let output_custom_role = PlugOptionsBuilder::create_output("theOutputPlug")
            .role(Some("customRole"))
            .build(&mut tether_agent)
            .unwrap();
        assert_eq!(output_custom_role.name(), "theOutputPlug");
        assert_eq!(output_custom_role.topic(), "customRole/any/theOutputPlug");

        let output_custom_id = PlugOptionsBuilder::create_output("theOutputPlug")
            .id(Some("customID"))
            .build(&mut tether_agent)
            .unwrap();
        assert_eq!(output_custom_id.name(), "theOutputPlug");
        assert_eq!(output_custom_id.topic(), "tester/customID/theOutputPlug");

        let output_custom_both = PlugOptionsBuilder::create_output("theOutputPlug")
            .role(Some("customRole"))
            .id(Some("customID"))
            .build(&mut tether_agent)
            .unwrap();
        assert_eq!(output_custom_both.name(), "theOutputPlug");
        assert_eq!(
            output_custom_both.topic(),
            "customRole/customID/theOutputPlug"
        );
    }

    #[test]
    fn input_manual_topics() {
        let mut tether_agent = TetherAgentOptionsBuilder::new("tester")
            .build()
            .expect("sorry, these tests require working localhost Broker");

        let input_all = PlugOptionsBuilder::create_input("everything")
            .topic(Some("#"))
            .build(&mut tether_agent)
            .unwrap();
        assert_eq!(input_all.name(), "everything");
        assert_eq!(input_all.topic(), "#");

        let input_nontether = PlugOptionsBuilder::create_input("weird")
            .topic(Some("foo/bar/baz/one/two/three"))
            .build(&mut tether_agent)
            .unwrap();
        assert_eq!(input_nontether.name(), "weird");
        assert_eq!(input_nontether.topic(), "foo/bar/baz/one/two/three");
    }
}
