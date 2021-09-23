#!/usr/bin/env node
const mqtt = require("async-mqtt");
const rc = require("rc");
const parse = require("parse-strings-in-object");
const { decode } = require("@msgpack/msgpack");
const { getLogger } = require("log4js");

const config = parse(
  rc("tetherSend", {
    loglevel: "info",
    protocol: "tcp",
    host: "localhost",
    port: 1883,
    topic: "#",
    username: "tether",
    password: "sp_ceB0ss!",
    path: "",
    format: {
      json: false,
    },
  })
);

const logger = getLogger("tether-receive");
logger.level = config.loglevel;

logger.debug(
  "tether-receive CLI launched with config",
  JSON.stringify(config, null, 2)
);

const setupSubsription = (client, topic) => {
  logger.debug(`Subscribing to topic "${topic}"`);
  client.subscribe(topic);
  client.on("message", (topic, message) => {
    try {
      const decoded = decode(message);
      logger.info(
        `received message on topic "${topic}": \n${JSON.stringify(decoded)}\n`
      );
    } catch (error) {
      logger.error("Could not decode message:", { message, error });
    }
  });
};

const main = async () => {
  const { protocol, host, port, username, password, path } = config;

  const url = `${protocol}://${host}:${port}${path}`;

  logger.info("Connecting to MQTT broker @", url, "...");

  try {
    const client = await mqtt.connectAsync(url, { username, password });
    logger.info("...connected OK");
    setupSubsription(client, config.topic);
  } catch (e) {
    logger.fatal("could not connect to MQTT broker:", e);
  }
};

main();
