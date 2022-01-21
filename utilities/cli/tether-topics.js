#!/usr/bin/env node
const mqtt = require("async-mqtt");
const rc = require("rc");
const parse = require("parse-strings-in-object");
const { getLogger } = require("log4js");

let topics = [];

const config = parse(
  rc("tetherTopics", {
    loglevel: "info",
    protocol: "tcp",
    host: "tether-io.dev",
    port: 1883,
    username: "tether",
    password: "sp_ceB0ss!",
    path: "",
    json: {
      enabled: false,
      commaSeparated: true,
      enclosingBrackets: true,
      includeTopics: true,
      includeTimestamps: true,
    },
  })
);

const logger = getLogger("tether-receive");
logger.level = config.json.enabled ? "fatal" : config.loglevel;

logger.debug(
  "tether-receive CLI launched with config",
  JSON.stringify(config, null, 2)
);

const main = async () => {
  const { protocol, host, port, username, password, path } = config;

  const url = `${protocol}://${host}:${port}${path}`;

  logger.info("Connecting to MQTT broker @", url, "...");

  try {
    const client = await mqtt.connectAsync(url, { username, password });
    logger.info("...connected OK");
    setupSubsription(client);
  } catch (e) {
    logger.fatal("could not connect to MQTT broker:", e);
  }
};

const setupSubsription = (client) => {
  client.subscribe("#");
  client.on("message", (topic, message) => {
    const match = topics.find((t) => t === topic);
    if (!match) {
      topics.push(topic);
      logger.info("Found topic", topic);
      logger.debug(`Current list (${topics.length} topics)`, topics);
    }
  });
};

main();
