# MessagePack experiments for Tether 2

It's important to note some key differences from Tether 1

- Protocol is MQTT (simplified compared to AMQP which is RabbitMQ specific) This in turn means:
  - Exchange is set to default `amq.topic`. A different default exchange has to be configured in the mqtt plugin, and cannot be specified on the client side.
  - The NodeJS base agent can/should actually use `mqtt` library (same as browser), rather than `amqplib`
    s

TODO:

- createOutput returns an Output object, but createInput does not... should be consistent?
