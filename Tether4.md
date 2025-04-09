"Tether Edition 2025, aka Tether 4"

# Breaking changes

The default will be TWO part topics, with THREE part topics optional.

The "ID" part was always optional; now this is put last and does NOT need to be included if unused. No need for inserting a word such as "any" into the topic.


For Output Plugs (publishing), the topic will be constructed as follows:
- agentRole/chanelName
- agentRole/chanelName/optionalID

For Input Plugs (subscribing), the topic will be constructed as follows:
- agentRole/chanelName/# (matches "no ID" part and "ID part included")
- agentRole/chanelName/optionalID (will only match when ID part is matched)

The main practical difference between a "topic" and a "Channel" (previously "plug") is simply that a Channel is expected to match ONLY ONE TYPE OF MESSAGE. So, a single MQTT Client may have multiple subscriptions, but we ensure that the correct messages are matched with the correct Channel when received, by applying our additional Tether Complaint Topic (TCT) matching pattern.

## Cleaning up
Unused "utilities" and the "explorer" will be removed.

## Proposed Terminology Changes
- Rename "ThreePartTopic" to "TetherCompliantTopic" (TCT).
- Use "Channel" instead of "Plug". This was an old proposal from the beginning of the project, and arguably makes much more sense since it clearly defines the intention to filter everything by the "type" of message expected to be sent or received.
- Instead of "InputPlug" and "OutputPlug", the word order will be reversed to "ChannelReceiver" and "ChannelSender". This reflects the idea that a "Channel" is a single thing, but may have multiple "ChannelReceivers" and "ChannelSenders" at either end.
- Instead of "publishing", we can simply talk about "sending", i.e. `channel.send()` rather than `plug.publish()`
- The term "receiving" will be preferred, but "subscribe" can still be used when actually relevant (especially internally)
- The folder "base_agent" will be renamed to "lib". There is no class inheritance thing happening in the library, anyway.

## New examples
Check that the React example works and is up to date!
Svelte example, please!
TouchDesigner (Python) and ESP32 (C++) examples should include both sending and receiving examples.
An example of integration from P5JS would be a good idea. Optionally, Cables.GL as well.

## Versioning
Unfortunately, the JS/TS package had already been bumped to v3 earlier. This means that we are actually on "Tether 2" up till v3.2.x and change to the very-different "Tether 3" from v3.3, supposedly a "minor version".
Rust and Python packages could both move to a "v3" properly, however, and it seems most meaningful to use the correct MAJOR version when importing the Tether Base Agent in any language.

Should we bump to "Tether 4" instead? (Reminds me of Angular 2->4 transition!)
