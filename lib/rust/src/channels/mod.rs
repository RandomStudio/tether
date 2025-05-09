pub mod definitions;
pub mod receiver;
pub mod sender;
pub mod tether_compliant_topic;

#[cfg(test)]
mod tests {
    use crate::{
        agent::builder::TetherAgentBuilder,
        receiver::ChannelReceiver,
        tether_compliant_topic::{parse_channel_name, TetherCompliantTopic, TetherOrCustomTopic},
        ChannelDef, ChannelDefBuilder, ChannelReceiverDefBuilder,
    };

    #[test]
    fn receiver_match_tpt() {
        let tether_agent = TetherAgentBuilder::new("tester")
            .auto_connect(false)
            .build()
            .expect("failed to create Agent");

        let channel = tether_agent.create_receiver::<u8>("testChannel").unwrap();

        let channel_def = channel.definition();

        assert_eq!(&channel_def.name, "testChannel");
        assert_eq!(channel_def.generated_topic(), "+/testChannel/#");
        assert_eq!(
            parse_channel_name("someRole/testChannel"),
            Some("testChannel")
        );
        assert_eq!(
            parse_channel_name("someRole/testChannel/something"),
            Some("testChannel")
        );
        assert!(channel.matches(&TetherOrCustomTopic::Tether(
            TetherCompliantTopic::new_three("dummy", "testChannel", "#")
        )));
        assert!(!channel.matches(&TetherOrCustomTopic::Tether(
            TetherCompliantTopic::new_three("dummy", "anotherChannel", "#")
        )));
    }

    #[test]
    fn receiver_match_tpt_custom_role() {
        let tether_agent = TetherAgentBuilder::new("tester")
            .auto_connect(false)
            .build()
            .expect("failed to create Agent");

        let channel_def = ChannelReceiverDefBuilder::new("customChannel")
            .role(Some("customRole"))
            .build(tether_agent.config());

        let channel = tether_agent
            .create_receiver_with_def::<u8>(channel_def)
            .expect("failed to create Channel");

        assert_eq!(channel.definition().name, "customChannel");
        assert_eq!(
            channel.definition().generated_topic(),
            "customRole/customChannel/#"
        );

        assert!(channel.matches(&TetherOrCustomTopic::Tether(
            TetherCompliantTopic::new_three("customRole", "customChannel", "#")
        )));
        assert!(channel.matches(&TetherOrCustomTopic::Tether(
            TetherCompliantTopic::new_three("customRole", "customChannel", "andAnythingElse")
        )));
        assert!(!channel.matches(&TetherOrCustomTopic::Tether(
            TetherCompliantTopic::new_three("customRole", "notMyChannel", "#"),
        ))); // wrong incoming Channel Name
        assert!(!channel.matches(&TetherOrCustomTopic::Tether(
            TetherCompliantTopic::new_three("someOtherRole", "customChannel", "#")
        ))); // wrong incoming Role
    }

    #[test]
    fn receiver_match_custom_id() {
        let tether_agent = TetherAgentBuilder::new("tester")
            .auto_connect(false)
            .build()
            .expect("failed to create Agent");

        let channel_def = ChannelReceiverDefBuilder::new("customChanel")
            .id(Some("specificID"))
            .build(tether_agent.config());

        let channel = tether_agent
            .create_receiver_with_def::<u8>(channel_def)
            .expect("failed to create Channel");

        assert_eq!(channel.definition().name, "customChanel");
        assert_eq!(
            channel.definition().generated_topic(),
            "+/customChanel/specificID"
        );
        assert!(channel.matches(&TetherOrCustomTopic::Tether(
            TetherCompliantTopic::new_three("anyRole", "customChanel", "specificID",)
        )));
        assert!(channel.matches(&TetherOrCustomTopic::Tether(
            TetherCompliantTopic::new_three("anotherRole", "customChanel", "specificID",)
        ))); // wrong incoming Role
        assert!(!channel.matches(&TetherOrCustomTopic::Tether(
            TetherCompliantTopic::new_three("anyRole", "notMyChannel", "specificID",)
        ))); // wrong incoming Channel Name
        assert!(!channel.matches(&TetherOrCustomTopic::Tether(
            TetherCompliantTopic::new_three("anyRole", "customChanel", "anotherID",)
        ))); // wrong incoming ID
    }

    #[test]
    fn receiver_match_both() {
        let tether_agent = TetherAgentBuilder::new("tester")
            .auto_connect(false)
            .build()
            .expect("failed to create Agent");

        let channel_def = ChannelReceiverDefBuilder::new("customChanel")
            .role(Some("specificRole"))
            .id(Some("specificID"))
            .build(tether_agent.config());

        let channel = tether_agent
            .create_receiver_with_def::<String>(channel_def)
            .expect("failed to create Channel");

        assert_eq!(channel.definition().name, "customChanel");
        assert_eq!(
            channel.definition().generated_topic(),
            "specificRole/customChanel/specificID"
        );
        assert!(channel.matches(&TetherOrCustomTopic::Tether(
            TetherCompliantTopic::new_three("specificRole", "customChanel", "specificID",)
        )));
        assert!(!channel.matches(&TetherOrCustomTopic::Custom(
            "specificRole/notMyChannel/specificID".into()
        ))); // wrong incoming Channel Name
        assert!(!channel.matches(&TetherOrCustomTopic::Custom(
            "specificRole/customChanel/anotherID".into()
        ))); // wrong incoming ID
        assert!(!channel.matches(&TetherOrCustomTopic::Custom(
            "anotherRole/customChanel/anotherID".into()
        ))); // wrong incoming Role
    }

    #[test]
    fn receiver_match_custom_topic() {
        let tether_agent = TetherAgentBuilder::new("tester")
            .auto_connect(false)
            .build()
            .expect("failed to create Agent");

        // Note alternative way of constructing channel with inline ChannelReceiverDefBuilder:
        let channel = tether_agent
            .create_receiver_with_def::<u8>(
                ChannelReceiverDefBuilder::new("customChannel")
                    .override_topic(Some("one/two/three/four/five"))
                    .build(tether_agent.config()),
            )
            .expect("failed to create Channel");

        assert_eq!(channel.definition().name(), "customChannel");
        // it will match on exactly the same topic:
        assert!(channel.matches(&TetherOrCustomTopic::Custom(
            "one/two/three/four/five".into()
        )));

        // it will NOT match on anything else:
        assert!(!channel.matches(&TetherOrCustomTopic::Custom("one/one/one/one/one".into())));
    }

    #[test]
    fn receiver_match_wildcard() {
        let tether_agent = TetherAgentBuilder::new("tester")
            .auto_connect(false)
            .build()
            .expect("failed to create Agent");

        // let channel = tether_agent
        //     .create_receiver_with_definition::<bool>(
        //         ChannelReceiverDefBuilder::new("everything")
        //             .override_topic(Some("#")) // fully legal, but not a standard Three Part Topic)
        //             .build(&tether_agent),
        //     )
        //     .expect("failed to create Channel");
        //

        // Note somehwat convoluted "direct ChannelReceiver::new" creation method
        let channel: ChannelReceiver<'_, u8> = ChannelReceiver::new(
            &tether_agent,
            ChannelReceiverDefBuilder::new("everything")
                .override_topic(Some("#")) // fully legal, but not a standard Three Part Topic)
                .build(tether_agent.config()),
        )
        .expect("failed to create Channel");

        assert_eq!(channel.definition().name(), "everything");

        // Standard TPT will match
        assert!(channel.matches(&TetherOrCustomTopic::Tether(
            TetherCompliantTopic::new_three("any", "chanelName", "#")
        )));

        // Anything will match, even custom incoming
        assert!(channel.matches(&TetherOrCustomTopic::Custom(
            "one/two/three/four/five".into()
        )));
    }
}
