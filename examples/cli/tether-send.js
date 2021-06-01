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
  })
);

const run = async () => {
  const { protocol, host, port } = config;

  const url = `${protocol}://${host}:${port}`;

  console.log("Connecting to MQTT broker @", url, "...");

  const client = await mqtt.connectAsync(url);

  console.log("...connected OK");

  const { topic, message } = config;

  const encoded = encode(JSON.parse(message));

  client.publish(topic, Buffer.from(encoded));

  // TODO: could have an input loop for new messages
  client.end();
};

run();