"Tether Edition 2025, aka Tether 4"

# Breaking changes

The default will be TWO part topics, with THREE part topics optional.

The "ID" part was always optional; now this is put last and does NOT need to be included if unused. No need for inserting a word such as "any" into the topic.


For Output Plugs (publishing) NOW CHANNEL SENDERS, the topic will be constructed as follows:
- agentRole/chanelName
- agentRole/chanelName/optionalID

For Input Plugs (subscribing) NOW CHANNEL RECEIVERS, the topic will be constructed as follows:
- agentRole/chanelName/# (matches "no ID" part and "ID part(s) included")
- agentRole/chanelName/optionalID (will only match when ID part is matched)

The main practical difference between a "topic" and a "Channel" (previously "plug") is simply that a Channel is expected to match ONLY ONE TYPE OF MESSAGE. So, a single MQTT Client may have multiple subscriptions, but we ensure that the correct messages are matched with the correct Channel when received, by applying our additional Tether Complaint Topic (TCT) matching pattern. The libraries (particularly typed languages such as TypeScript and Rust) should try to encourage (if not enforce) this practice.

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

## JS (TS) changes
Apart from the terminology changes, the following are important to note:
- Encoding and decoding of messages happens AUTOMATICALLY BY DEFAULT now, so there is no need to import and use `encode` and `decode`
- The `.send` function automatically encodes, and uses generics! No need to encode first (and applications must be careful NOT to encode before sending). `.sendRaw` is provided as an alternative if the end-user prefers to encode themselves
- The EventEmitter class extension business has been removed, so callbacks for receiving messages are handled manually (internally). From the end-user perspective, everything looks very much the same, with the exception that `.on("message", cb)` will AUTOMATICALLY DECODE first, and tries to use generics! The end-user currently has no direct option to decode messages themselves, any more.

### "Ownership" of Channels

Returning to an older concept (pre-version "3" for TS/JS), the end-user can decide whether to manage Channel Sender/Receivers "themselves" (essentially, keep ownership of the Channel instances) or allow the TetherAgent instance to handle this for them.

Practically, it means that you could create a ChannelReceiver in two ways:

1. `const sender = agent.getReceiver("numberValues")` : Let TetherAgent own the ChannelReceiver
2. `const sender = await ChannelReceiver.create(agent, "numberValues")` : Let user/application own the ChannelReceiver

In the case of (1.), assigning the "same" receiver (by name) will in fact NOT initiate the creation of a new ChannelReceiver or the subscription to the MQTT Broker. `const sender2 = agent.getReceiver("numberValues")` will return the existing, previously-created, ChannelReceiver object. This is because the TetherAgent keeps track of a list of ChannelReceivers (and, similarly, ChannelSenders) internally.

This type of functionality is quite useful in a web application, particularly in frontend frameworks where the "same channel" could be re-used in various places / UI components. Also, the order of "creating" and "getting" a particular channel might be complicated to manage in many cases; this way, it doesn't matter in which order this happens.

This functionality probably makes little sense for a Rust application, by contrast, where ownership needs to be much more explicit, and the end-user developer should be expected to manage the order of creating/mutating/deleting things more carefully.

## Rust changes
Apart from the terminology changes, the following are important to note:
- `agent.send` used to assume an already-encoded payload, while `.encode_and_send` did auto-encoding. Now, `.send` is the auto-encoding version and additional `.send_raw` and `.send_empty` functions are provided. It is VERY important that the new `.send` will actually (incorrectly!) accept already-encoded payloads, because `&[u8]` is ALSO `T: Serialize`! So applications using the new version must be carefully checked to ensure that things are not (double) encoded before sending! It does mean that the end-user application no longer needs to do encoding/decoding themselves.

The term "OptionsBuilder" suffix has now been replaced with the much simpler "Builder", so we have simply:
- TetherAgentBuilder
- ChannelSenderBuilder
- ChannelReceiverBuilder

Even better, the ChannelSenderBuilder/ChannelReceiver builder do not **have** to be used in all cases, since both ChannelSender and ChannelReceiver objects can be constructed via the Tether Agent object itself, i.e.

- `tether_agent::create_sender`
- `tether_agent::create_receiver`

All that needs to be provided, in the default cases, is the name and the type. For example:
- `tether_agent.create_sender::<u8>("numbersOnly")` creates a ChannelSender called "numbersOnly" which will automatically expect (require) u8 payloads

The TypeScript library is now set up to mirror this as well (also, optional!), where it means having to pass fewer arguments (and also allows the TetherAgent to "own" the Channels, which is **not** the case in the Rust library).
