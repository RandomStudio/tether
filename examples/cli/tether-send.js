const mqtt = require("async-mqtt");
const rc = require("rc");
const parse = require("parse-strings-in-object");
const { encode } = require("@msgpack/msgpack");

const config = parse(
  rc("tetherSend", {
    protocol: "tcp",
    host: "localhost",
    port: 1883,
    topic: `tetherCli/unknown/dummy`,
    message: '{ "hello": "world" }',
    username: "tether",
    password: "sp_ceB0ss!",
  })
);

const run = async () => {
  const { protocol, host, port, username, password } = config;

  const url = `${protocol}://${host}:${port}`;

  console.log("Connecting to MQTT broker @", url, "...");

  const client = await mqtt.connectAsync(url, { username, password });

  console.log("...connected OK");

  const { topic, message } = config;

  const parsedMessage = JSON.parse(message);

  const encoded = encode(parsedMessage);

  client.publish(topic, Buffer.from(encoded));

  console.log("sent", parsedMessage, "on topic", topic);

  // TODO: could have an input loop for new messages
  client.end();
};

run();
