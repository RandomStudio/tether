# Tether

Instead of trying to find (or build) the One True Best Creative Coding Tool, we decided to find a way to make the tools we were already using (and the ones we didn't even know about yet) work together.

By using Tether, we can approach digital art/media installations as **distributed systems**, applying a **publish / subscribe pattern** and **event-based programming** to coordinate the independent pieces of software and hardware that commonly comprise these systems.

Specifically, Tether is a standardised way of using existing, well established technologies such as [MQTT](https://mqtt.org/) (for messaging) and [MessagePack](https://msgpack.org/index.html) (for serialised data).

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
2. At least one Tether Agent capable of _publishing_ messages encoded in MessagePack format
3. At least one Tether Agent capable of _receiving (subscribing to)_ messages, decoding in MessagePack format

### GUI

If you'd like to use a graphical / desktop application to test out Tether, try:

- [Tether Egui](https://github.com/RandomStudio/tether-egui)

Tether Egui acts as both an Agent that publishes (by default, as "gui") but also subscribes to all topics (by default) and tries to decode the MessagePack contents. So you can use a single instance of Tether Egui to simulate an entire (very basic) Tether system, provided you are also running an MQTT Broker on your system.

### CLI

Alternatively, command-line utilities are provided [here](./utilities/tether-utils) (with instructions for installing them)

- `tether send`: by default, publishes messages as the Agent "tether-send"
- `tether receive`: subscribes to messages on all topics (by default) and tries to decode the MessagePack payload of each one

You can use `tether send` in combination with `tether receive` to simulate a minimal Tether system.

---

## Understanding Tether

The Tether approach is about abstracting the underlying hardware and software so that everything from the point of view of the "system" is an **Agent** that can communicate using standardised **Messages**.

### Agents

The best way to describe a Tether System is not to start with message brokers, topics and protocols; it's best to start by talking about **Agents**.

Agents in a Tether system are very much like Actors in an [Actor Model](https://en.wikipedia.org/wiki/Actor_model) for concurrent computing.

Generally, every piece of software in a Tether system is an Agent. An Agent should have a single well-defined role, e.g. "publish tracking data" or "output sounds".

Agents can be written in various programming languages and they could be running on the same host device or multiple devices (PCs, microcontrollers) - but they all communicate to each other in the same standardised way.

### Plugs

**Agents** are at the _top_ or most _general_ level of the Tether system hierarchy. At the _bottom_ or most _fine-grained_ level, we try to separate "types" of messages - we call these **Plugs**.

Plugs define the "end point" of each message type - either its destination or its source.

Agents might be _sending_ one or more "types" of messages - we call these **Output Plugs**. Other Agents might be _receiving_ one or more "types" of message - we call these **Input Plugs**. An Agent might have Output Plug(s) or Input Plug(s) or a combination of both. One agent can therefore have multiple Plugs.

Although we can't really enforce it, a single Plug should only send/receive messages of a single "type" - format or schema. For example, if your TrackingData Plug always sends 2-element arrays of numbers that represent `[x,y]` coordinates in the range `[0;1]`, then you shouldn't expect to see a message like `{ "position": { x: 0, y: 1 }}` or an empty message.

### Publish/Subscribe

In a Tether System, we don't connect Agents to each other _directly_.

While it is _usual_ for each Output Plug to have at least one (or more) corresponding Input Plug at the "other end", we don't enforce this or assume that this is the case before things can start running.

Instead, we rely on the so-called [publish/subscribe pattern](https://en.wikipedia.org/wiki/Publish%E2%80%93subscribe_pattern) ("Pub/Sub"), a well-established approach that allows for flexible network topologies and an "event-based" architecture. It is most commonly used in distributed systems; for example, [Amazon AWS describes the benefits](https://aws.amazon.com/pub-sub-messaging/benefits/) as follows:

> In modern cloud architecture, applications are decoupled into smaller, independent building blocks that are easier to develop, deploy and maintain. Publish/Subscribe (Pub/Sub) messaging provides instant event notifications for these distributed applications.

> The Publish Subscribe model enables event-driven architectures and asynchronous parallel processing, while improving performance, reliability and scalability.

In our experience, this is a good fit for on-site digital art/media "installations" as well, since these are often composed of multiple pieces of software and hardware devices. These individual pieces (we call them **Agents**, remember?) are often completely independent. They run in parallel, and they might start and stop at different times. They may be written in different programming languages and some of them might not allow any direct programmatic integration at all (SDKs or APIs) - but they all need to communicate with one another somehow.

Careful coordination of all the parts is what allows us to build systems that function as complete "experiences" - robust and seamless. A common messaging system allows us to pick and choose the hardware and software best suited for each task, and handle the communication issues separately.

In a Tether System, as with all Pub/Sub systems, messages are organised under "topics". We are a little more strict about how we name these topics, so that the concepts of Agents and Plugs are represented in the topics themselves.

Finally, note that this is a _push_ messaging system. Therefore, no polling is required, and Agents need to be prepared to handle messages whenever they come in.

### Input/Output Roles vs Input/Output Plugs?

Sometimes the use of the words **Input** and **Output** can be confusing depending on the context. "An agent whose role is mostly input will have mostly output plugs" ... what?

> 💡 TIP: Think from the point of view of the Tether System. Everything coming in from the outside world is "input" and everything going to the outside world is "output".

![Input vs Output in a diagram](./docs/input-brain-output.png)

Here are some examples that should clear up the terminology.

- An Agent that **gets** data from the outside world (e.g. sensor input or user control)
  - ...Will **get** the data "in" from other devices and systems
  - ...Will convert this data into Tether Messages, and mostly **publish** these messages **to** the Tether System
  - ...It will therefore have mostly (or exclusively) **OUTPUT PLUGS**
  - Nevertheless, we say its **role** is essentially **INPUT**
- An Agent that **outputs** graphics, lights, sound, data, etc.
  - ...Will **send** data "out" to other devices, displays, specialised software, etc.
  - ...Will mostly **subscribe** to Tether Messages **from** other parts of the Tether System, e.g instructions on what to do next, events that need to be responded to, or state that must be represented
  - ...It will therefore have mostly (or exclusively) **INPUT PLUGS**
  - ...Nevertheless, we say its role is essentially **OUTPUT**

Some Agents will do a bit of both "input" _and_ "output". See the section on [Agent Roles](#part-1-agent-or-role) for more detail and examples of these.

## Formally defining a Tether System

To make a Tether system, the following conventions (A, B, C) are applied:

- A: All communication passes through a MQTT message broker
- B: Apply a standardised, 3 part topic route convention (`agent/id/plug`)
- C: MessagePack is used for the contents of the messages

### A: The MQTT Broker

#### Why MQTT specifically?

- Widely-supported standard for Pub/Sub messaging, especially for IOT and distributed systems
- Supports flexible publish/subscribe patterns (one to one, one to many, many to many, etc.) and efficient queueing of messages to facilitate this
- Works via standard TCP sockets as well as WebSocket; therefore usable from many different environments and programming languages, as well as the browser

#### Where is my broker?

For our purposes, a single broker per installation/system is typically sufficient. It's convenient to use Docker to provide a pre-configured instance, so anything that can run Docker (a Mac, a Raspberry Pi, a Linux PC, a Windows PC) is suitable.

The broker could be running on the same machine as the other agents, in which case all connections can point at `localhost` - the lowest-latency option, and useful for a development / testing / simulation environment.

The broker could, in theory, be hosted on the Internet, but for our installations it usually makes sense to have it on a local network "on premises" - this guarantees easy accessibility by other devices/hosts and low latency.

Or mix it up a little: a powerful machine might host the broker as well as some other Agents (which connect to `localhost`), while a few dedicated machines run other Agents and connect to the broker using an IP address on the LAN. As a convention, TCP connections are accepted at port `1883` and websocket at `15675`.

#### Performance considerations

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

### B: The Three Part Topic

MQTT Topics can be of varying length (`one`, `one/two` and `something/something/foo/bar/baz` are all valid topics with 1, 2 or 5 parts respectively).

Tether is all about naming things carefully. So we keep things standard.

Topics in Tether are therefore always composed of exactly 3 parts: `agent` / `id` / `plug`

Or, to be more descriptive: `"agent role"` / `"id or group"` / `"plug name"`. Notice that the order of the parts moves from most general (the Agent) to most specific (the Plug) as you go from left to right.

#### Topic Parts

##### Part 1: Agent or Role

Each Agent is expected to have a single "role" in the system. A short indication/naming of the role is used as the top level of the topic hierarchy.

> 💡 TIP: If the distinction between a mostly "Output" role and "Input" role seems confusing, review the section on [Input/Output Roles vs Input/Output Plugs](#inputoutput-roles-vs-inputoutput-plugs).

Some examples of Agent roles:

- Mostly **INPUT** Role -> Output Plugs:
  - `"lidar2d"` for LIDAR data, e.g. [tether-lidar-rs](https://github.com/RandomStudio/tether-rplidar-rs). Note that the underlying hardware, SDK and even programming language could differ, but from the point of view of the Tether system the role is the same because the messages look the same.
  - `"lidar-person-counter"` for presence detection, e.g. [lidar-person-counter](https://github.com/RandomStudio/lidar-person-counter)
  - `"gui"` for user interface control, e.g. [tether-egui](https://github.com/RandomStudio/tether-egui)
  - `"poseDetection"` for tracking people
  - `"videoColourFinder"` for detecting dominant colours from a webcam, e.g. [Tether Colourfinder](https://github.com/RandomStudio/tether-colourfinder-web)
  - `"midi"` for turning MIDI input from a controller or keyboard into standardised Tether messages, e.g. [tether-midi-mediator](https://github.com/RandomStudio/tether-midi-mediator/tree/main)
  - `"scheduler"` for emitting off/on notifications for processes on a schedule, e.g [tether-scheduler](https://github.com/RandomStudio/tether-scheduler)
- Mostly Input Plugs -> **OUTPUT** Role:
  - `"soundscape"` for output of audio driven by remote messages, e.g. [tether-soundscape-rs](https://github.com/RandomStudio/tether-soundscape-rs)
  - `"visualisation"` could cover a range of screen-based graphical output, either via a browser frontend or some native application

Some Agents do both "input" and "output". Data from multiple sensors might need to be integrated, for example. Or a complex combination of state management, timed animation, events, decisions might be turned into new Tether Messages.

- Both **INPUT** and **OUTPUT** Roles
  - `"brain"` is a very common agent role in most of our installations. This is a process dedicated to managing state and responding to events (e.g. from sensors or time-based) and generating other events (controlling output, starting timelines, etc.). Usually these are very customised for the given project.
  - `"lidarConsolidation"` for taking sensor input (in this case, one or more "lidar2d" agents) and running clustering + perspective transformation algorithms, then outputting nicely normalised "tracking" data. See [Tether Lidar2D Consolidator](https://github.com/RandomStudio/tether-lidar2d-consolidation-rs)

#### Part 2: ID or Group

Every agent should have a single role, but in many cases there may be multiple instances of the same type of agent - to distinguish these as necessary, the second level of the topic hierarchy is therefore an **identifier**.

Sometimes you don't need to distinguish between instances - either because there is only one (common for a "brain" role) or because you intend to treat the messages the same regardless of their origin/destination. In this case, you can default to an identifier such as `"any"`.

Other times, it's useful to distinguish instances. For example, the identifier part of the topic could be a string based on:

- Serial numbers of LIDAR devices; these could be useful for saving position, calibration and other configuration data specific to each device.
- MAC ID (unique network address) from microcontrollers. This could be a convenient way to distinguish instances without having to hardcode information on each device.
- A grouping that makes sense in your installation. It might be useful to have multiple instances share the same identifier.

#### Part 3: Plug

Any given Agent might publish one or more distinct types of messages. This last level of the topic hierarchy we name the **plug**.

The concept of a "plug" is simply a convention, such that:

- Only one type of message is expected to be published on that topic
- The plug name attempts to describe the contents, utility or purpose of the messages that will be published there
- From the point of view of a given Agent, a plug is either
  - an **Input Plug**: subscribe to a particular topic, including wildcard/pattern matches (see below)
  - an **Output Plug**: publish on a particular topic
  - ..._never both_

### Topic pattern matching

Let's put these "parts" together by describing how MQTT matches actual topics (for published messages) to topic patterns (for subscribing).

MQTT topics are broken up by zero or multiple forward-slash `/` characters. In Tether systems, we **always** have three-part topics, hence `agent/id/plug`.

Topics subscriptions can use wildcards. Most importantly:

- `#` = "match all". Can be used on its own or at the _end_ of a sequence.
  - `#` matches all topics, without exception
  - `brain/#` would match `brain/any/events` as well as `brain/any/metrics` but also `brain/foyerArea/events`
- `+` = "match part". Substitute exactly **one** part of a topic. One or more of these symbols can be used at _any level_.
  - In Tether we often use a pattern like `+/+/somePlugName` to subscribe to messages of the type `somePlugName` on _any agent role_ and _any ID_.
  - Other times, you may be interested in messages from a specific role AND plug name, but you don't care about the ID. So `scheduler/+/metrics` would help you to get `metrics` messages from some agent with the role `scheduler`, but ignore those coming from `brain/any/metrics` or `scheduler/any/events`

The conventions are often applied automatically by the various Base Agents we provide in this repo for Your Favourite Programming Language™️. For example:

- You typically provide the Agent **Role** and **ID** just once when creating a new Tether Agent instance. (The latter might default to `any` if you don't provide one).
- We typically require you to only provide the **plug name** when creating an **Output Plug**
  - The Agent **Role** and **ID** are added to the topic automatically, so a plug named `"colours"` will automatically publish messages on a topic like `colourDetection/any/colours`
  - You may also override the topic if you wish, but be careful of breaking the conventions!
  - Remember that you cannot _publish_ on a topic with wildcards!
- We typically require you to only provide the **plug name** when creating an **Input Plug**
  - By default, we assume you don't care to distinguish by **role** or **ID**, so we automatically subscribe to a topic like `+/+/whateverPlugNameYouProvided`
  - Of course you can override this by providing your own topic string, but don't break the conventions!

In the JS Base Agent, we create an InputPlug or OutputPlug object that provides callbacks such as `.onMessage` (for InputPlug) and `.publish` (for OutputPlug).

In other languages, it may make more sense to use utility functions that can parse the topic to give you **role**, **ID** or just **plugName** depending on your matching requirements.

### C: The MessagePack Payload

We chose MessagePack because it represents a good compromise in terms of performance, message size and the ability to structure data (e.g. in nested objects with named keys, and/or arrays), but without needing a schema in order to serialise/deserialise data. Has most of the obvious advantages of JSON but more efficient: MessagePack data is encoded directly as bytes rather than a "String".

Unlike JSON, you can even provide "bare" data instead of nested objects. For example, you can send a message with a single boolean (`true` or `false`) value instead of something more verbose like `{ "state": true }`. What you lose in explicitness (you ought to use the plug name to describe the messages well, in this case) you gain in terse data representation: MessagePack will literally encode such a message as a single byte!

---

## Goals and benefits of using Tether

As long as client applications conform to the standards outlined here, they will be able to function as Tether Agents, publishing and subscribing to messages in a Tether system.

The aim is to make it quick and easy to get messaging working within a distributed system, even with very diverse programming languages, hardware and software applications. The combination of MQTT and MessagePack means that a Tether system is just about the _easiest and quickest_ way to get parts of a distributed system talking to each other. It requires very little code, minimal APIs and very little network configuration.

Other approaches (HTTP requests, websocket servers, OSC, etc.) may sometimes appear easier to reach for in certain circumstances, but typically do not offer the flexibility of a "pub/sub" messaging system or a structured (but very transparent) data structure in the messages.

The technology can be integrated very easily in everything from websites to microcontrollers to game engines. Translating in and out from other protocols/transports (e.g. MIDI, OSC, serial data) is convenient enough that software which is "not Tether-native" can be plugged in without much effort.

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

### System diagram examples

#### A simple example

This example features a microcontroller connected to a passive infrared (PIR) sensor, publishing a boolean "true" (motion detected) or "false" (timeout) message, which stops and starts a video on screen depending on the received state.

What the system diagram might look like:
![System View](./docs/simple-system.png)

> This looks simple enough. You could even plug the PIR straight into the Raspberry Pi, and have one piece of software handling the electronic input as well as playing the video?

Some potential drawbacks:

- Maybe video playback and data reading don't run so well on a single process with the Raspberry Pi and something like Python
- Where are the components going to be placed relative to each other? As soon as cable distances change it might make sense to have the input running on an Arduino and the output on the Pi, and how will they communicate?
- What if you you need to use multiple sensors and/or multiple outputs?
- None of this software would be reusable "as is", because it was all quite specific to this one scenario

What this could look like as a Tether system:
![Tether View](./docs/simple-tether.png)

> This looks more complicated, but ultimately is more flexible and reusable

The separation of the `brain` (custom Agent for this installation) and the `videoPlayer` (presumably a generic remote-control video player Agent) has some advantages:

- The "videoPlayer" and "presenceDetector" Agents could be reusable in other projects
- Multiple video player agents could run for multiple screens; the brain could stay on one of these Pi's or even its own dedicated "server"
- The MQTT Broker and the Brain could run on a single, dedicated "server" Pi
- Everything is networked so cable run distances and alternative placements are no longer problematic
- The individual Agents could be tested and/or simulated independently by replicating the messages the send or receive - no need to run the whole system with all its hardware

#### A more complex example

This example features 11 Agents, running on 7 or more independent hosts/devices. Input is via multiple LIDAR sensors but some remote control also comes in via an iPad interface of some kind (could be a web page). Output includes screen graphics, a scent diffuser controlled by a relay switch, audio and some light animations.

What your system diagram looks like:
![System View](./docs/complex-system.png)

> There are multiple protocols and transports. Communication needs to happen between some parts and not others, and these may be on different hosts. Some data needs to pass through multiple stages before becoming useful. How should these parts find each other? What protocols should they use? How will you troubleshoot network problems or mistakes in configuration?

What it looks like from the point of view of the Tether system:
![Tether View](./docs/complex-tether.png)

> Everything connects to a single hub: the MQTT Broker. Agents only need to concern themselves with publishing certain messages and/or subscribing to certain messages. The protocols and hardware "behind" the Agents are invisible to the Tether system. The choice of hosts is much more flexible: e.g. `brain` and `screenOutput` could run on the same machine (if it's powerful enough) or two (heavy graphics might need to run on a dedicated machine), and this would make no difference to the Tether system at all!

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
