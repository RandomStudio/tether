const mqtt = require("async-mqtt");
const rc = require("rc");
const parse = require("parse-strings-in-object");
const { encode } = require("@msgpack/msgpack");
const { getLogger } = require("log4js");

const config = parse(
  rc("tetherSend", {
    loglevel: "info",
    protocol: "tcp",
    host: "localhost",
    port: 1883,
    topic: `tether-send/unknown/dummy`,
    message: '{ "hello": "world" }',
    username: "tether",
    password: "sp_ceB0ss!",
    path: "",
  })
);

const logger = getLogger("tether-receive");
logger.level = config.loglevel;

logger.debug(
  "tether-send CLI launched with config",
  JSON.stringify(config, null, 2)
);

const sendMessages = (client, message, topic) => {
  try {
    const parsedMessage = JSON.parse(message);
    const encoded = encode(parsedMessage);

    client.publish(topic, Buffer.from(encoded));
    logger.info("sent", parsedMessage, "on topic", topic);
  } catch (error) {
    logger.error("Could not parse or send message:", { message, error });
  }
};

const main = async () => {
  const { protocol, host, port, username, password, path } = config;

  const url = `${protocol}://${host}:${port}${path}`;

  logger.info("Connecting to MQTT broker @", url, "...");

  try {
    const client = await mqtt.connectAsync(url, { username, password });
    logger.info("...connected OK");
    sendMessages(client, config.message, config.topic);

    // TODO: should loop / REPL before closing
    client.end();
  } catch (e) {
    logger.fatal("could not connect to MQTT broker:", e);
  }
};

main();
