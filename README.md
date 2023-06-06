# Tether

A standardised way of using [MQTT](https://mqtt.org/) (for message publishing and subscribing) and [MessagePack](https://msgpack.org/index.html) (for serialised data in message payloads).

Also, a set of tools and conventions built around these standards, to make it easy to set up distributed systems where more than one piece of software needs to communicate with others in a coordinated way, whether on the same host or multiple hosts/devices.

## Structure of this repository

- `base_agent`: Not a requirement (you can simply follow Tether Conventions) but providing convenience in the following programming languages:
  - `js`: Base agent in Javascript, suitable for both NodeJS and browser environments. [JS Base Agent README](./base_agent/js/README.md)
  - `cpp`: Base agent in C++11, using CMake for build/install automation. [C++ Base Agent README](./base_agent/cpp/README.md)
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

## Why MQTT

- Widely-supported standard, especially for IOT and distributed systems
- Supports flexible publish/subscribe patterns (one to one, one to many, many to many, etc.)
- Works via TCP sockets as well as Websocket; therefore usable from many different environments and programming languages, as well as the browser

## Why MessagePack

A good compromise in terms of performance, message size and the ability to structure data (e.g. in nested objects with named keys, and/or arrays), but without needing a schema in order to serialise/deserialise data. Has most of the advantages of JSON but more efficient.

## What defines a Tether Agent

To be part of a Tether system, the following conventions are applied:

- A standardised, 3 part topic route convention (`agent-role/grouping-or-id/plug-name`)
- A single MQTT broker with a known IP address or hostname that is reachable by all other agents
- The expectation that MessagePack will be used for the contents of the messages

The concept of a "plug" is simply a convention, such that:

- Only one type of message is expected to be published on that topic
- The plug name attempts to describe the contents, utility or purpose of the messages that will be published there
- From the point of view of a given Agent, a plug is either an Input (subscribe to a particular topic) or an Output (publish on a particular topic), never both

As long as client applications conform to the above standards, they will be able to publish and read messages in a Tether system. The aim is to make it quick and easy to get messaging working within a distributed system, even with very diverse programming languages, hardware and software applications.

Various tools, naming conventions, permissions and choices of architecture can be built on top of this system. There is no guarantee that every Tether-like system will work perfectly or behave in the same way, but at least the hard part - distributed messaging - is "solved" so that developers can concentrate on more interesting concerns.

## Goals

The combination of these two simple and robust technologies (MQTT and MessagePack) means that a Tether system is just about the _easiest and quickest_ way to get parts of a distributed system talking to each other. It requires very little code, minimal APIs and very little network configuration.

Other approaches (websocket servers, OSC, etc.) may sometimes appear easier to reach for in certain circumstances, but typically do not offer the flexibility of a "pub/sub" messaging system or a structured (but very transparent) data structure in the messages.

The technology can be used very easily in everything from websites to microcontrollers to game engines. Translating in and out from other protocols/transports (e.g. MIDI, OSC, serial data) is easy enough that software which is "not Tether-native" can be plugged in without much effort.
