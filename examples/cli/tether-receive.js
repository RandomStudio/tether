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
    username: "tether",
    password: "sp_ceB0ss!",
    path: "",
  })
);

const run = async () => {
  const { protocol, host, port, username, password, path } = config;

  const url = `${protocol}://${host}:${port}${path}`;

  console.log("Connecting to MQTT broker @", url, "...");

  const client = await mqtt.connectAsync(url, { username, password });

  console.log("...connected OK");

  const { topic } = config;

  client.subscribe(topic);
  client.on("message", (topic, message) => {
    try {
      const decoded = decode(message);
      console.log(
        `received message on topic "${topic}": ${JSON.stringify(decoded)}`
      );
    } catch (e) {
      console.log("Could not decode message:", { message, e });
    }
  });
};

run();
