"Tether Edition 2025, aka Tether 3"

# Breaking changes

The standard will be TWO part topics, not THREE part topics.

The "ID" part was always optional; now this is put last and does NOT need to be included if unused. No need for inserting a word such as "any" into the topic.


For Output Plugs (publishing), the topic will be constructed as follows:
- agentRole/plugName
- agentRole/plugName/optionalID

For Input Plugs (subscribing), the topic will be constructed as follows:
- agentRole/plugName/# (matches "no ID" part and "ID part included")
- agentRole/plugName/optionalID (will only match when ID part is matched)

The main practical difference between a "topic" and a "Plug" is simply that a Plug is expected to match ONLY ONE TYPE OF MESSAGE. So, a single MQTT Client may have multiple subscriptions, but we ensure that the correct messages are matched with the correct Input Plug when received, by applying our additional Standard Tether topic matching pattern.