# Tether Base Agent

Tether is a standardised MQTT+MessagePack system for inter-process communication, created, maintained and actively in use by [Random Studio](https://random.studio).

Read more about Tether in the [main Tether repo](https://github.com/RandomStudio/tether).

## Use in NodeJS / Browser

This package is JavaScript base agent for the browser and NodeJS.

The JS Base Agent can be used by both NodeJS scripts (TCP or Websocket) and web pages (Websocket only). We provide some easy defaults (import `NODJES` or `BROWSER`) for each environment.

Example of browser defaults, but override the host (and hostname):
```
import { TetherAgent, BROWSER } from "tether-agent";

const agent = await TetherAgent.create("dummy", {
  brokerOptions: {
    ...BROWSER,
    host: "192.168.27.100",
  }
);
```

Example of NodeJS defaults, with no overrides:
```
import { TetherAgent, NODEJS } from "tether-agent"

const agent = await TetherAgent.create("dummy", {
  brokerOptions: BROKER_DEFAULTS.nodeJS
);
```

Actually, NodeJS defaults are just the defaults, so even easier:
```
import { TetherAgent } from "tether-agent";

const agent = await TetherAgent.create("dummy");
```


## Features

The Base Agent is intentionally a very thin layer on top of standard MQTT + MsgPack. We merely add some conventions on top of what MQTT can already provide. See the [main Tether documentation](https://github.com/RandomStudio/tether) for more information on our approach.

For the JS/TypeScript Base Agent specifically, we encapsulate the functionality of a Tether Agent in a class, which retains:

- The details for the MQTT Broker, and methods to connect and disconnect it
- The "Role" (required) and "ID" (optional) for this agent

For Channels, we encapsulate sending and receiving in the classes ChannelSender and ChannelReceiver respectively:
- When creating instances of either ChannelSender or ChannelReceiver, you pass a reference to the Agent so that some sensible defaults are set up for publishing or subscribing
- ChannelSender creation is synchronous (use `new ChannelSender(agent, "channelName")`) while ChannelReceiver creation should be asynchronous (use `await ChannelReceiver.create(agent, "channelName)`) because subscription needs to be requested behind the scenes

We also provide the `encode` and `decode` functions from the [@msgpack/msgpack JS library](https://www.npmjs.com/package/@msgpack/msgpack) dependency to allow easy encoding and decoding of payloads.

## Usage

You can find some ES6-style Javascript sample code in `/examples/nodejs` and some React with Typescript sample code in `/examples/react-ts`.

The basic steps are usually something similar to the following:

1. Create a Tether Agent instance with `const agent = await TetherAgent.create("someRole")` - this connects automatically to the MQTT broker, by default
2. Create Channel Senders that you need with `const mySender = new OutputPlug("somePlugName")`
3. Create Channel Receivers that you need with `const myReceiver = await InputPlug.create(agent, "somePlugName")`

For Output Plugs, you can send messages like this:

```
await mySender.send(encode({ foo: "bar }));
```

For Input Plugs, the subscription is set up automatically on `.create` but you need to handle incoming messages, e.g.:

```
myReceiver.on("message", (payload, _topic) => {
  const decoded = decode(payload);
});
```

## Usage with Vite
Vite seems to have a little difficulty with the "Buffer" package which this library depends on. The following addition to the `vite.config.js` file appears to resolve this:

```
resolve: {
    alias: {
      Buffer: "buffer",
      mqtt: "mqtt/dist/mqtt.js",
    },
  },
```