# Tether Base Agent

Tether is a standardised MQTT+MessagePack system for inter-process communication, created, maintained and actively in use by [Random Studio](https://random.studio).

Read more about Tether in the [main Tether repo](https://github.com/RandomStudio/tether).

## Use in NodeJS / Browser

This package is JavaScript base agent for the browser and NodeJS.

The JS Base Agent can be used by both NodeJS scripts (TCP or Websocket) and web pages (Websocket only). We provide some easy defaults (import `BROKER_DEFAULTS`) for each environment:

```
// Example of browser defaults, but override the host (and hostname)
const agent = await TetherAgent.create("dummy", {
  brokerOptions: {
    ...BROKER_DEFAULTS.browser,
    host: "192.168.27.100",
  }
);
```

```
// Example of NodeJS defaults, with no overrides
const agent = await TetherAgent.create("dummy", {
  brokerOptions: BROKER_DEFAULTS.nodeJS
);
```

## Features

The Base Agent is intentionally a very thin layer on top of standard MQTT. We merely add some conventions on top of what MQTT can already provide. See the [main Tether documentation](https://github.com/RandomStudio/tether) for more information on our approach.

For the JS Base Agent specifically, we encapsulate the functionality of a Tether Agent in a class, which retains:

- The details for the MQTT Broker, and methods to connect and disconnect it
- The "Role" and "ID" for this agent
- The Input and Output "Plugs" which are used to subscribe and publish respectively

We also provide the `encode` and `decode` functions from the [@msgpack/msgpack JS library](https://www.npmjs.com/package/@msgpack/msgpack) dependency to allow easy encoding and decoding of payloads.

## Usage

You can find some ES6-style Javascript sample code in `/examples/nodejs` and some React with Typescript sample code in `/examples/react-ts`.

The basic steps are usually something similar to the following:

1. Create a Tether Agent instance with `const agent = await TetherAgent.create("someRole")` - this connects automatically to the MQTT broker, by default
2. Create Output Plug(s) that you need with `const myOutputPlug = new OutputPlug("somePlugName")`
3. Create Input Plug(s) that you need with `const myInputPlug = await InputPlug.create(agent, "somePlugName")`

For Output Plugs, you can send messages like this:

```
await myOutputPlug.publish(encode({ foo: "bar }));
```

For Input Plugs, the subscription is set up automatically on `.create` but you need to handle incoming messages, e.g.:

```
myInputPlug.on("message", (payload, _topic) => {
  const decoded = decode(payload);
});
```
