![Tether Logo Vertical](./docs/tether-logo-horizontal.png)

# Simply == Connect == Everything

Instead of trying to find (or build) the One True Best Creative Coding Tool, we decided to find a way to make the tools we were already using (and the ones we didn't even know about yet) work together in a sensible way.

As people who love making things,  we wanted to be careful to avoid the temptation of "re-inventing the wheel". We're also not great with lots of rigid structure.

Tether is about providing tools and processes that help us spend more time on the important (and fun, and creative) things. Tether is our attempt to add a little bit of structure, but not too much.

## Architecture
By using Tether, we can approach digital art/media installations as **distributed systems**, applying a **publish / subscribe pattern** and **event-based programming** to coordinate the independent pieces of software and hardware that commonly comprise these systems.

This facilitates a wide variety of system architectures in practice. Tether makes it easy to build very simple systems with one-way communcation: one application talking to another. It's also possible to implement very rigidly-structured systems comprising strict request/response sequences. Or any wild topology you can come up with (multiple producers, multiple consumers, etc.).

## Freedom, but with sensible defaults
Tether applies some light _standardisation_ of some existing, well established technologies such as [MQTT](https://mqtt.org/) (for messaging) and [MessagePack](https://msgpack.org/index.html) (for serialised data). By picking a standard, we can provide a set of libraries and tools to make this infrastructure simple to use.

Tether makes our software more reliable and easier to re-use.

---

## Table of Contents

- [Quick start](#quick-start)
  - [GUI](#gui)
  - [CLI](#cli)
- [Understanding Tether](#understanding-tether)
  - [Agents](#agents)
  - [Plugs](#plugs)
  - [Publish/Subscribe](#publishsubscribe)
  - [Input/Output Roles vs Input/Output Plugs?](#inputoutput-roles-vs-inputoutput-plugs)
- [Formally defining a Tether System](#formally-defining-a-tether-system)
  - [A. The MQTT Broker](#a-the-mqtt-broker)
    - [Why MQTT specifically?](#why-mqtt-specifically)
    - [Where is my Broker?](#where-is-my-broker)
    - [Performance considerations](#performance-considerations)
    - [Retained messages](#retained-messages)
  - [B. The Three Part Topic](#b-the-three-part-topic)
    - [Topic parts](#topic-parts)
      - [Part 1: Agent/Role](#part-1-agent-or-role)
      - [Part 2: ID/Group](#part-2-id-or-group)
      - [Part 3: Plug](#part-3-plug)
    - [Topic pattern matching](#topic-pattern-matching)
  - [C. The MessagePack Payload](#c-the-messagepack-payload)
- [Goals and benefits of using Tether](#goals-and-benefits-of-using-tether)
  - [Debugging and Troubleshooting](#debugging-and-troubleshooting)
  - [System diagram examples](#system-diagram-examples)
    - [A simple example](#a-simple-example)
    - [A more complex example](#a-more-complex-example)
- [Structure of this repository](#structure-of-this-repository)

---

## Quick start

The most basic Tether system comprises:

1. An MQTT broker (see [brokers/README](brokers/README.md) for instructions on setting one up)
2. At least one Tether Agent capable of _sending_ messages encoded in MessagePack format
3. At least one Tether Agent capable of _receiving_ messages, decoding in MessagePack format

### GUI

If you'd like to use a graphical / desktop application to test out Tether, try:

- [Tether Egui](https://github.com/RandomStudio/tether-egui)

Tether Egui lets you create some UI "widgets" for numbers, colours, strings and more. You'll be sending Tether messages in no time. It also provides a full suite of utilities for:

- exploring and monitoring all the messages in a Tether-based system
- recording and playback

### CLI

Alternatively, command-line utilities are provided [here](./utilities/tether-utils) (with instructions for installing them)

For example, the command `tether receive` will subscribe to messages on all topics (by default), decode and print out each one.

You can run `tether receive` in one terminal window, and `tether send` in another - and just like that, you have a working Tether system!

---

## Understanding Tether

The Tether approach is about abstracting the underlying hardware and software so that everything from the point of view of the "system" is an **Agent** that can communicate using standardised **Messages**. We direct each type of Message down a separate **Channel**.

### Messages
For Tether we have decided to use the MsgPack format for messages. This has all the advantages of a "schemaless" or "untyped" format such as JSON, namely:

- You don't have to know the message format in order to decode a message (but it helps!)
- You can name things when you think it's helpful to have self-describing data (with key-value pairs) such as `{x: 0,5, y: 0}`, but you can also be very terse when you think it's appropriate, e.g. `[0.5, 0]`
- It's well-supported for serialisation (encoding) and deserialisation (decoding) across lots of different programming languages and environments

MsgPack has a few advantages over JSON, however:
- It's much more compact over the wire, stripping out characters such as brackets, spaces and quotation marks
- It is usually faster to encode/decode than JSON, which is especially important in constrained environments (e.g. microcontrollers) where you don't have a lot of RAM and CPU power

It's important to note that in Tether systems we often cannot **enforce** strict typing of messages, but we highly recommend it. If you are using TypeScript, for example, make proper types (interfaces) for your messages and share these between Agents that send and Agents that receive those messages.

### Agents

Agents in a Tether system are very much like Actors in an [Actor Model](https://en.wikipedia.org/wiki/Actor_model) for concurrent computing.

Generally, every piece of software in a Tether system is an Agent. An Agent should have a single well-defined role, e.g. "get sensor data" or "play sounds".

Agents can be written in various programming languages and they could be running on the same host device or multiple devices (PCs, microcontrollers). They could be in the same room or scattered across the world. But they all communicate to each other in the same standardised way.

In the Tether Base Agents (libraries for various languages), we abstract the underlying connection to the MQTT Broker within the concept of an Agent. This means we can provide some sensible defaults so you don't have to worry about setting IP addresses, ports, protocols (TCP vs WebSocket), or paths. You can often just name the Agent and the connection is made automatically!

It's really important to **name** Agents properly. If the Agent is made to control lights, name it "lightingController"; if it's meant to detect presence, name it "presenceDetector". This helps you (and future you, and other developers) understand where all these messages come from!

### Channels

A **Channel** can be thought of as the pipe, connector or cable between two (or more) Agents. In a Tether system, we promise to send only one type of Message through that pipe. Whether it's a true/false value, a number, an array or a nested object of key-value pairs, or even nothing at all (e.g. a trigger), Messages in a Channel should always be the same format.

(The underlying technology, MQTT, does not dictate anything about whether messages with the same "topic" have to have a particular type or format. Our Tether libraries typically can't enforce this, either. But if we all agree to restrict ourselves to one message type for each Channel, then everything becomes easier to manage.)

Agents can be _sending_ one or more "types" of Messages. Each Message type is send using a dedicated (and re-usable) **Channel Sender**.

Agents might be _receiving_ one or more "types" of Message. For receiving, each Message type is received using a dedicated **Channel Receiver**.

An Agent might have one or more Channel Senders and/or one or more Channel Receivers; or a combination of both. One Agent can therefore be connected into the system via one or more Channels.

It's really important to **name** Channels properly. If the channel contains x, y coordinates of something, call it "positions"; if it is meant to send cues for lighting scenes, name it "scenes". This helps not only to describe the likely contents of the messages but helps to "advertise" the capabilities of an Agent in the system: if you can see that an Agent named "lidar2d" is publishing Channels named "positions" but also "presenceEvents" then you have a good idea what to expect.

---

## Recommended Structure for Tether-based Projects

### Order of events
Create the Agent (and implicitly connect), asynchronously
Create one or more Channel Sender instances (synchronously), passing the Agent instance so it gets sensible defaults
Create one or more Channel Receiver instances (asynchronously, because subscription needs to be requested from the MQTT broker), passing the Agent instance so it gets sensible defaults

Call a sending method (e.g. `.send`) on the Channel Sender (or the Agent instance; this can vary between languages) whenever you need to send a message in your program. The message body can be empty. If it is not empty, encode the message in MsgPack format before sending.

Attach a callback (e.g. `myChannelReceiver.on("message", (payload) => { ... })`) and handle the messages. Or, if your language does not support this model, poll the Agent instance periodically and check each incoming message against the Channel Receiver definition you set up earlier, to see which one it belongs to (without having to inspect the topic string yourself!).

### Custom vs generic
Custom = brain

Below are some examples of Agents.

Mostly **sending** role:
- `"lidar2d"` for LIDAR data, e.g. [tether-lidar-rs](https://github.com/RandomStudio/tether-rplidar-rs) (in Rust) or [tether-rplidar](https://github.com/RandomStudio/tether-rplidar) (in Python). Note that the underlying hardware, SDK and even programming language could differ, but from the point of view of the Tether system the role is the same because the messages look the same.
- `"lidar-person-counter"` for presence detection, e.g. [lidar-person-counter](https://github.com/RandomStudio/lidar-person-counter)
- `"gui"` for user interface control, e.g. [tether-egui](https://github.com/RandomStudio/tether-egui)
- `"poseDetection"` for tracking people
- `"videoColourFinder"` for detecting dominant colours from a webcam, e.g. [Tether Colourfinder](https://github.com/RandomStudio/tether-colourfinder-web)
- `"midi"` for turning MIDI input from a controller or keyboard into standardised Tether messages, e.g. [tether-midi-mediator](https://github.com/RandomStudio/tether-midi-mediator/tree/main)
- `"scheduler"` for emitting off/on notifications for processes on a schedule, e.g [tether-scheduler](https://github.com/RandomStudio/tether-scheduler)
- `"fusionIMU"` for emitting orientation and acceleration data from a microcontroller equipped with an Intertial Measurement Unit (IMU), e.g. [Tether Wireless IMU](https://github.com/RandomStudio/tether-imu)

Mostly **receiving** role:
- `"soundscape"` for audio driven by remote messages, e.g. [tether-soundscape-rs](https://github.com/RandomStudio/tether-soundscape-rs)
- `"rendering"` could cover a range of screen-based graphical output, either via a browser frontend or some native application

Many Agents do both sending and receiving, and tend to live "in the middle" of the message flow. Data from multiple sensors might need to be integrated, for example. Or a complex combination of state management, timed animation, events, decisions might be turned into new Tether Messages.

Both **sending** and **receiving** Roles:
- `"brain"` is a very common agent role in most of our installations. This is a process dedicated to managing state and responding to events (e.g. from sensors or time-based) and generating other events (controlling output, starting timelines, etc.). Usually these are very customised for the given project.
- `"lidarConsolidation"` for taking sensor input (in this case, one or more "lidar2d" agents) and running clustering + perspective transformation algorithms, then outputting nicely normalised "tracking" data for other Tether Agents to use. See [Tether Lidar2D Consolidator](https://github.com/RandomStudio/tether-lidar2d-consolidation-rs)


### Publish/Subscribe Architecture

In a Tether System, we don't connect Agents to each other _directly_.

While it is _usual_ for each Output Plug to have at least one (or more) corresponding Input Plug at the "other end", we don't enforce this or assume that this is the case before things can start running.

Instead, we rely on the so-called [publish/subscribe pattern](https://en.wikipedia.org/wiki/Publish%E2%80%93subscribe_pattern) ("Pub/Sub"), a well-established approach that allows for flexible network topologies and an "event-based" architecture. It is most commonly used in distributed systems; for example, [Amazon AWS describes the benefits](https://aws.amazon.com/pub-sub-messaging/benefits/) as follows:

> In modern cloud architecture, applications are decoupled into smaller, independent building blocks that are easier to develop, deploy and maintain. Publish/Subscribe (Pub/Sub) messaging provides instant event notifications for these distributed applications.

> The Publish Subscribe model enables event-driven architectures and asynchronous parallel processing, while improving performance, reliability and scalability.

In our experience, this is a good fit for on-site digital art/media "installations" as well, since these are often composed of multiple pieces of software and hardware devices. These individual pieces (we call them **Agents**, remember?) are often completely independent. They run in parallel, and they might start and stop at different times. They may be written in different programming languages and some of them might not allow any direct programmatic integration at all (SDKs or APIs) - but they all need to communicate with one another somehow.

Careful coordination of all the parts is what allows us to build systems that function as complete "experiences" - robust and seamless. A common messaging system allows us to pick and choose the hardware and software best suited for each task, and handle the communication issues separately.

In a Tether System, as with all Pub/Sub systems, messages are organised under "topics". We are a little more strict about how we name these topics, so that the concepts of Agents and Plugs are represented in the topics themselves.

Finally, note that this is a _push_ messaging system. Therefore, no polling is required, and Agents need to be prepared to handle messages whenever they come in.


### Performance considerations

Message Brokers such as Mosquitto are designed for extremely high throughput (tens of thousands of messages per second) and availability, so the broker itself is seldom a bottleneck. Using wired connections (ethernet) where possible and reducing unnecessary network traffic are good practices. Having a dedicated "server" host which runs the broker - and nothing else - is not required but may be useful in some scenarios.

MQTT provides QOS (Quality of Service) levels to help you balance **reliability** vs **latency** for publishing and/or subscribing. There are three levels:

- At most once (0)
- At least once (1)
- Exactly once (2)

We default to QOS level 1 most of the time, but level 0 can be useful for high-frequency data where you don't mind missing a message or two, and level 2 can be useful for critical messages (state or events) that don't happen often but need solid guarantees for delivery.

Read more about QOS [here](https://www.hivemq.com/blog/mqtt-essentials-part-6-mqtt-quality-of-service-levels/).


#### Retained messages

One very useful feature of MQTT is the ability to mark messages as "retained" - usually for as long as the Broker itself is running.

This can be useful for storing configuration or state information, almost like a database or a web server:

- Whenever state or config data changes, you only need to publish it once. And you can do this at any time, not needing to worry which Agents may or may not be "listening" at that moment.
- Agents subscribed to the topic will get the latest version of the data as soon as they subscribe, e.g. on (re)connection. The Broker re-sends the message automatically.
- The latest version of the data (message) can be read at any time, without affecting any other subscribers (the message will not be "consumed").


### Retaining the Goals and Benefits of Tether

As long as client applications conform to the standards outlined here, they will be able to function as Tether Agents, sending and receiving to messages in a Tether system.

The aim is to make it quick and easy to get messaging working within a distributed system, even with very diverse programming languages, hardware and software applications. The combination of MQTT and MessagePack means that a Tether system is just about the _easiest and quickest_ way to get parts of a distributed system talking to each other. It requires very little code, minimal APIs and very little network configuration.

Other approaches (HTTP requests, websocket servers, OSC, etc.) may sometimes appear easier to reach for in certain circumstances, but typically do not offer the flexibility of a "pub/sub" messaging system or a structured (but very transparent) data structure in the messages.

The technology can be integrated very easily in everything from websites to microcontrollers to game engines. Translating in and out from other protocols/transports (e.g. MIDI, OSC, serial data) should be convenient enough that software which is "not Tether-native" can be plugged in without much effort.

### Debugging and Troubleshooting

Tether systems are super easy to debug - when compared to the usual "hacked together" distributed system - because all messages can be subscribed to without affecting other Agents. Messages do not get "consumed", because the MQTT Broker is responsible for duplicating and queueing things behind the scenes.

Use [Tether Egui](https://github.com/RandomStudio/tether-egui) to monitor, decode and simulate messages with an easy-to-use desktop app.

![Tether Egui screenshot](./docs/tether-egui.gif)

Or use the [Tether CLI](https://github.com/RandomStudio/tether/tree/main/utilities/cli) utilities to:

- Subscribe to all messages passing through the MQTT Broker without affecting anything: `tether receive`
- List all known Agents, Topics and Plugs on the system: `tether topics`
- Record data from one or multiple Agents (even a whole system!) using `tether record` and `tether playback`

![Tether Topics CLI screenshot](./docs/tether-topics.png)

The ability to use simulated data (including timing information!) when developing systems that would otherwise require a lot of specialised hardware and software to be running all at once.

---

## Formally defining a Tether System

To make a Tether system, the following conventions (A, B, C) are applied:

- A: All communication passes through a MQTT message broker
- B: Apply a standardised, 2 or 3 part Tether Compliant Topic convention: `agent/channel` or `agent/channel/id`
- C: MessagePack is used for the contents of the messages

### A: The MQTT Broker

#### Why MQTT specifically?

- Widely-supported standard for Pub/Sub messaging, especially for IOT and distributed systems
- Supports flexible publish/subscribe patterns (one to one, one to many, many to many, etc.) and efficient queueing of messages to facilitate this
- Works via standard TCP sockets as well as WebSocket; therefore usable from many different environments and programming languages, as well as the browser

#### Where is my broker?

For our purposes, a single broker per installation/system is typically sufficient. It's convenient to use Docker to provide a pre-configured instance, so anything that can run Docker (a Mac, a Raspberry Pi, a Linux PC, a Windows PC) is suitable.

The broker could be running on the same machine as the other agents, in which case all connections can point at `localhost` - the lowest-latency option, and useful for a development / testing / simulation environment.

The broker can be hosted on the Internet, but for our installations it usually makes sense to have it on a local network "on premises" - this guarantees easy accessibility by other devices/hosts and low latency.

Or mix it up a little: a powerful machine might host the broker as well as some other Agents (which connect to `localhost`), while a few dedicated machines run other Agents and connect to the broker using an IP address on the LAN.

As a convention, we always configure our MQTT Brokers as follows:
- TCP connections are accepted at port `1883`
- Websocket connections are accepted at port `15675` at the path`/ws`
- Username is `tether` and password is `sp_ceB0ss!`

The above conventions are used in default Base Agent setups in all languages, and should be overridden explicitly only when needed.

### B: Tether-Compliant Topic

MQTT Topics can be of varying length (`one`, `one/two` and `something/something/foo/bar/baz` are all valid topics with 1, 2 or 5 parts respectively). Tether is all about naming things carefully. So we keep things standard.

Topics in Tether are therefore always composed of exactly 2 **required** parts plus 1 **optional** part: `agent` / `channel` [/ `id`]

Or, to be more descriptive: `"agent role"` / `"channel name"`/ `"id, grouping or other distinguishing name"`.

#### Topic Parts

##### Part 1 (Required): Agent or Role

Each Agent is expected to have a single "role" in the system. A short indication/naming of the role is used as the top level of the topic hierarchy.

There is usually one Agent instance (and therefore one MQTT client connection) for each application or process, but it is perfectly possible to use multiple Agent instances in a single application if this makes sense (e.g. for simulating messaging between two distinct Agents).

#### Part 2 (Required): Channel

Any given Agent might publish one or more distinct types of messages. In the second part of a Tether-compliant topic hierarchy we name the **Channel**.

The concept of a Channel is simply a convention, such that:

- Only one type of message is expected to be published on the topic with this Channel name
- The Channel name attempts to describe the format, utility or purpose of the messages that will be published there
- From the point of view of a given Agent, a Channel is defined by - and accessed via - either:
  - an **Channel Sender**: used to publish messages, with a topic completely defined by 2 or 3 parts (no wildcards)
  - an **Channel Receiver**: used to subscribe, with a topic that typically includes wildcard characters such as `+` and/or `#`

#### Part 3 (Optional): ID or Group

Every Agent should have a single role, but in many cases there may be multiple instances of the same type of Agent. In some cases, there might be overlap in the naming of Channels (lighting "scenes" and soundscape "scenes" might share the same name, for example. To distinguish these as necessary, the third level of the topic hierarchy is therefore an **identifier, grouping or distinguishing name**.

Often you don't need to distinguish between instances - either because there is only one (common for a "brain" role) or because you intend to treat the messages the same regardless of their origin/destination. In this case, you don't need to define this part anywhere.

Other times, it's useful to distinguish instances. For example, the identifier part of the topic could be a string based on:

- Serial numbers of LIDAR devices; these could be useful for saving position, calibration and other configuration data specific to each device.
- MAC ID (unique network address) from microcontrollers. This could be a convenient way to distinguish instances without having to hardcode information on each device.
- A grouping that makes sense in your installation. It might be useful to have multiple instances share the same identifier, e.g. `environmentSensor/temperature/inside` and `environmentSensor/humidity/inside`.
- Distinguishing between Channels that otherwise have the same name, e.g. lighting cues and video cues could be separated as `lightingController/cues/forLights` vs `videoPlayer/cues/forVideo`

The optional ID part can be specified at various levels. There are some subtleties here, intended to provide intuitively sensible results in practice.

For Sending:
- If you provide an ID when the **Agent** instance is created, this will be appended to all messages that are **sent** automatically
- If you provide an ID when a **Channel Sender** instance is created, it will override anything that might have been specified for the Agent, and will be appended on all messages that are sent

The implication of the above is that any Tether system may feature a combination of messages with and without the "ID" part, e.g. `sensor/values` and `sensor/values/specialIdentifier` - the intention is that BOTH of these topics still belong to the same Channel "values" and it is up to the Receiver to determine whether the distinguishing part is relevant/required.

For Receiving:
- If you provide an ID when the **Agent** instance is created, it will automatically be appended to any Channel Receiver instances created
- If you provide an ID when a **Channel Receiver** instance is created, it will override anything that might have been specified at the Agent level
- If no ID is specified either for the Agent or the Channel Receiver instance, then the last part of the topic subscription will be a wildcard `#`, i.e. `myRole/myChannel/#` which will match BOTH topics that exclude the ID part (2 parts only) AND topics that do include the ID part (3 parts)

### Topics pattern matching vs Channels
Topics and Channels are related but distinct terms in Tether.

MQTT provides its own topic pattern matching, which is specifically designed to ensure that clients only receive messages that they intend to "subscribe" to. Tether takes this a little further because we want to confine certain message types to certain "channels", which has the following implications:
- Often we want to subscribe to messages on multiple topics (multiple channels) which in MQTT typically all come through the same, single client connection. We need to distinguish which message "belongs" to which Channel.
- We want to be able to optionally apply filtering by Agent, Channel or ID (or a combination of these) in a way that MQTT doesn't automatically facilitate.

In general, MQTT topics are broken up by zero or multiple forward-slash `/` characters. In Tether systems, we **always** have 2 or 3 part topics, hence `agent/channel` or `agent/channel/id`.

Topics subscriptions can use wildcards. Most importantly:

- `#` = "match all". Can be used on its own or at the _end_ of a sequence.
  - `#` matches all topics, without exception
  - `brain/events/#` would match `brain/events` as well as `brain/events/room1` but also `brain/foyerArea/room2`. This is useful for making Channel Inputs that can subscribe to messages from a single Channel but optionally filter by the ID part *but only if specified* by the end-user application.
- `+` = "match part". Substitute exactly **one** part of a topic. One or more of these symbols can be used at _any level_.
  - In Tether we often use a pattern like `+/someChannelName/#` to subscribe to messages of the type `someChannelName` on _any agent role_ and _any ID_ (or no ID!).

The conventions are often applied automatically by the various Base Agents we provide in this repo for Your Favourite Programming Language™️. For example:

- You typically provide the Agent **Role** and optionally the **ID** just once when creating a new Tether Agent instance
- We require you to only provide the **channel name** when creating a **Channel Sender**
  - The Agent **Role** and optional **ID** are added to the topic automatically, so a plug named `"colours"` will automatically publish messages on a topic such as `colourDetection/colours` if you provided no ID, or `colourDetection/colours/screen1` if you provided the ID
  - You may also override the topic if you wish, but be careful of breaking the conventions!
  - Remember that you can never _publish_ on a topic that includes wildcards!
- We require you to only provide the **channel name** when creating an **Channel Receiver**
  - By default, we assume you don't care to distinguish by **Role** or **ID**, so we automatically subscribe to a topic like `+/whateverPlugNameYouProvided/#`
  - The default topic generated above will match by Channel name only, ignoring Agent Roles and any ID parts that may or may not have been included
  - Of course you can override this by providing your own topic string, but don't break the conventions!

In the Javascript/TypeScript Base Agent, we create an instance of the ChannelSender or ChannelReceiver class. ChannelSender instances provide methods for sending messages (`.send`); ChannelReceiver instances provides callbacks (`.onMessage`).

In other languages, it may make more sense to use utility functions that can parse the topic to give you **channelName**, **role** or  **ID**  depending on your matching requirements. In Rust, we provide a `.matches(&topic)` function that automatically applies the Tether matching conventions by comparing any incoming topic to the Channel "definition"; thereby, matching Messages against the Channels they "belong" to.

### C: The MessagePack Payload

We chose MessagePack because it represents a good compromise in terms of performance, message size and the ability to structure data (e.g. in nested objects with named keys, and/or arrays), but without needing a schema in order to serialise/deserialise data. Has most of the obvious advantages of JSON but more efficient: MessagePack data is encoded directly as bytes rather than a "String".

Unlike JSON, you can even provide "bare" data instead of nested objects. For example, you can send a message with a single boolean (`true` or `false`) value instead of something more verbose like `{ "state": true }`. What you lose in explicitness (you ought to use the plug name to describe the messages well, in this case) you gain in terse data representation: MessagePack will literally encode such a message as a single byte!

---

## Structure of this repository

- `base_agent`: Not a requirement (you can simply follow Tether Conventions) but providing convenience in the following programming languages:
  - `js`: Base agent in Javascript, suitable for both NodeJS and browser environments. [JS Base Agent README](./base_agent/js/README.md)
  - `cpp`: Base agent in C++11, using CMake for build/install automation. [C++ Base Agent README](./base_agent/cpp/README.md)
  - `python`: Base agent in Python, tested with Python v3.9
  - `rs`: Base agent in Rust. This is a crate also [published on crates.io](https://crates.io/crates/tether-agent). You can also find the auto-generated [docs](docs.rs/tether-agent/0.5.2) via docs.rs. Examples, as per Rust conventions, are included in the examples subfolder; or go to the [README](./base_agent/rs/README.md)
- `explorer`: A proof-of-concept of a browser-based ("web application") agent which uses _both_ the JS base agent and pure-MQTT-client approaches to demonstrate input and output being passed via the browser
- `examples`
  - `nodejs`: A demo agent that uses the same JS base agent as the "explorer". It publishes messages on two separate topics every 3 seconds, and also decodes any messages it receives on the "browserData" Input Plug.
  - `arduino`: Demonstrating how a Tether-like "agent" (without needing a Base Agent) can be written for a microcontroller
- `utilities`:
  - `cli`: Sending and Receiving from the command-line. These utilities can be installed globally on the system (via npm) to be used to test, monitor and troubleshoot a Tether-based system, interacting with it in pure text.
- `brokers`: Docker Compose configurations for the following MQTT brokers:
  - `mosquitto`: Currently the preferred option for Tether - supports all MQTT features (including retained messages) and MQTT-over-websocket by default
  - `rabbitmq`: Uunfortunately does not support retained messages
  - `nanomq`: Lightweight broker, but has been known to occasionally become unresponsive
