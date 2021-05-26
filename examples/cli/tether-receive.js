const mqtt = require("async-mqtt");
const rc = require("rc");
const parse = require("parse-strings-in-object");
const { decode } = require("@msgpack/msgpack");

const config = parse(
  rc("tetherSend", {
    protocol: "tcp",
    host: "localhost",
    port: 1883,
    topic: "#",
  })
);

const run = async () => {
  const { protocol, host, port } = config;

  const url = `${protocol}://${host}:${port}`;

  console.log("Connecting to MQTT broker @", url, "...");

  const client = await mqtt.connectAsync(url);

  console.log("...connected OK");

  const { topic } = config;

  client.subscribe(topic);
  client.on("message", (topic, message) => {
    const decoded = decode(message);
    console.log(
      `received message on topic "${topic}": ${JSON.stringify(decoded)}`
    );
  });
};

run();
