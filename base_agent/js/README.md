# Tether Base Agent

Tether is a standardised MQTT+MessagePack system for inter-process communication, created, maintained and actively in use by [Random Studio](https://random.studio).

Read more about Tether in the [main Tether repo](https://github.com/RandomStudio/tether).

## Use in NodeJS / Browser

This package is JavaScript base agent for the browser and NodeJS.

The JS Base Agent can be used by both NodeJS scripts and web pages, with minor differences:

- In NodeJS, the default MQTT protocol over TCP socket works as normal, although it is possible to use websockets as well
- In the browser, only MQTT-over-Websocket will work, but you must make some changes to the standard configuration - see example below

```js
TetherAgent.create("myTetherAgent", {
  protocol: "ws",
  port: 15675,
  host: "localhost",
  path: "/ws",
});
```

In future, we might detect which environment javascript is running in, and switch to some sensible defaults without any further configuration needed by the end-user developer.

## Features

The Base Agent is intentionally a very thin layer on top of standard MQTT, so that Tether "agents" can interoperate without needing such a Base Agent class or library. This reduces the burden of having to port the Base Agent to other languages and platforms.

We merely add some convenience on top of what MQTT can already provide:

- Establish connections to the MQTT broker (the "Tether Server") using sensible/standardised defaults
- Collecting some standard information - the Tether Agent "type" and "id" - which can be used to automatically set up topics to publish on, without the user having to specify the entire route
- Defining Input and Output "Plugs" which are very thin abstractions on top of topics - see below
  - publish messages on Output Plugs without having to specify the topic each time
  - handle incoming messages from Input Plugs without having to distinguish topics for each message

We encapsulate the functionality of a Tether Agent in a class, which retains

- The details for the MQTT Broker, and methods to connect and disconnect it
- The "type" and "id" for this agent
- The Input and Output "Plugs" which are used to subscribe and publish respectively

Currently, we do _not_ include any built-in ways of encoding or decoding message contents via MessagePack, though arguably this should be included too.

## Plugs vs MQTT Topics

In the Tether Base Agent, a Plug is just a way of building an MQTT topic route in a standardised, Tether-like, way.

For publishing, this means that the user can specify only a "plug name" for creating an Output Plug, and the rest of the route is built up using the Agent "type" and "ID" already configured when the Tether Agent class was instantiated. This auto-generated topic can be overidden easily.

For subscribing, again the user can specify only a "plug name" for an Input Plug and the topic will, by default, be built using wildcards that match anything ending with that plug name. Again, this can be overidden easily when the developer has a need to subscribe to a more narrowly-defined topic, e.g. Agent ID, type or both.
