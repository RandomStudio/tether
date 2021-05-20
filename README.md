# MQTT+MessagePack POC for Tether 2

## Structure of this project

- `base_agent`
  - `js`: The only base agent implemented so far. You don't even have to use this (see a few examples below).
- `explorer`: A proof-of-concept of a browser-based agent which uses _both_ the JS base agent and pure-MQTT-client approaches to demonstrate input and output being passed via the browser
- `examples`
  - `cli`: Showing how little is actually needed to send and receive messages via MQTT+MsgPack
  - `nodejs`: A demo agent that uses the same JS base agent as the "explorer". It publishes messages on two separate topics every 3 seconds, and also decodes any messages it receives on the "browserData" Input Plug.
- `rabbitmq_docker`: A Dockerfile for building RabbitMQ with the MQTT and MQTT-web plugins enabled, and a docker-compose file to map the appropriate ports.

## Running RabbitMQ with websocket/mqtt support

A `docker-compose.yml` file (and corresponding `Dockerfile`) is available in `./rabbitmq_docker`. This installs the appropriate plugins (for web/MQTT) and sets up the port mappings needed.

Run with `docker-compose up` (attach, keep running in this shell) or `docker-compose up -d` (daemonise).

To keep it persistent (even between reboots) the following worked well:

- `restart: "unless-stopped" was added to the `docker-compose.yml`
- Made sure that Docker was enabled as a service in systemd: `sudo systemctl enable docker`
- Started the container (once!) using `docker-compose up -d`

## MQTT vs AMQP

It's important to note some key differences from Tether 1

- Protocol is MQTT (simplified compared to AMQP which is RabbitMQ specific) This in turn means:
  - Exchange is set to default `amq.topic`. A different default exchange has to be configured in the mqtt plugin, and cannot be specified on the client side.
  - The NodeJS base agent can/should actually use `mqtt` library (same as browser), rather than `amqplib`
  - MQTT uses / not . separators, and wildcards are different (see https://www.hivemq.com/blog/mqtt-essentials-part-5-mqtt-topics-best-practices/), e.g. `+` not `*` and `#` may only appear at the _end_ of topic/routing key (or is the only symbol)

Because MQTT does not specify the need for "exchanges" and "queues" (though RabbitMQ may use these internally), the subscription process is quite different. Subscription is on the level of the _client_ not the (channel+exhange+)queue.

This means that if we keep the concept of a defined "Input Plug" (which seems sensible, since this can then be queried/reported elsewhere) then a little bit of extra work is done to match incoming messages with the correct Input instance, since there is no inherent link between the subscription and the messages that come in. Incoming messages include a topic (a string) - that is all that is needed to match with the Input.

It also means that we cannot (and probably don't need to) stop the end-user ignoring the concept of an "Input Plug" altogether, listening for incoming messages and then breaking up and interpreting topic strings in order to redirect/handle messages as they see fit.

## TODOs

- [x] use MQTT library for both NodeJS and browser
- [x] use same base agent in both NodeJS and browser
- [x] createOutput returns an Output object, but createInput does not... should be consistent?
- [x] demonstrate a simple CLI client (send and receive)
- [ ] demonstrate microcontroller client (Arduino): without even a base agent, and no need for Plugs
