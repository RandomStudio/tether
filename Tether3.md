"Tether Edition 2025, aka Tether 3"

# Breaking changes

The default will be TWO part topics, with THREE part topics optional.

The "ID" part was always optional; now this is put last and does NOT need to be included if unused. No need for inserting a word such as "any" into the topic.


For Output Plugs (publishing), the topic will be constructed as follows:
- agentRole/plugName
- agentRole/plugName/optionalID

For Input Plugs (subscribing), the topic will be constructed as follows:
- agentRole/plugName/# (matches "no ID" part and "ID part included")
- agentRole/plugName/optionalID (will only match when ID part is matched)

The main practical difference between a "topic" and a "Plug" is simply that a Plug is expected to match ONLY ONE TYPE OF MESSAGE. So, a single MQTT Client may have multiple subscriptions, but we ensure that the correct messages are matched with the correct Input Plug when received, by applying our additional Standard Tether topic matching pattern.

## Cleaning up
Unused "utilities" and the "explorer" will be removed.

## Proposed Terminology Changes
- Rename "ThreePartTopic" to "TetherCompliantTopic" (TCT).
- Use "Channel" instead of "Plug". This was an old proposal from the beginning of the project, and arguably makes much more sense since it clearly defines the intention to filter everything by the "type" of message expected to be sent or received at either end.
- Instead of "InputPlug" and "OutputPlug", the word order will be reversed to "ChannelInput" and "ChannelOutput". This reflects the idea that a "Channel" is a single thing, but may have multiple "ChannelInputs" and "ChannelOutputs" at either end.

## New examples
TouchDesigner and ESP32 examples should include both publishing and subscribing examples.
