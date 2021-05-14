# MessagePack experiments for Tether 2

## Running RabbitMQ with websocket/mqtt support
A `docker-compose.yml` file (and corresponding `Dockerfile`) is available in `./rabbitmq_docker`. This installs the appropriate plugins (for web/MQTT) and sets up the port mappings needed.

Run with `docker-compose up` (attach, keep running in this shell) or `docker-compose up -d` (daemonise).


## MQTT vs AMQP

It's important to note some key differences from Tether 1

- Protocol is MQTT (simplified compared to AMQP which is RabbitMQ specific) This in turn means:
  - Exchange is set to default `amq.topic`. A different default exchange has to be configured in the mqtt plugin, and cannot be specified on the client side.
  - The NodeJS base agent can/should actually use `mqtt` library (same as browser), rather than `amqplib`

## TODOs

- use MQTT library for both NodeJS and browser
- createOutput returns an Output object, but createInput does not... should be consistent?
- demonstrate a simple CLI client (send and receive)
- demonstrate microcontroller client (Arduino)
