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

const setupSubsription = (client, topic) => {
  logger.info(`Subscribing to topic "${topic}"`);
  client.subscribe(topic);
  if (config.json.enabled && config.json.enclosingBrackets) {
    console.log("[");
  }
  let messageCount = 0;
  let startTime = null;
  client.on("message", (topic, message) => {
    if (startTime === null) {
      startTime = Date.now(); // Start timing from first message received
    }
    messageCount++;
    try {
      const decoded = decode(message);
      const { enabled, includeTopics, includeTimestamps } = config.json;
      if (enabled) {
        if (messageCount > 1) {
          process.stdout.write(",\n");
        }
        const jsonString = {
          ...decoded,
          topic: includeTopics ? topic : undefined,
          timestamp: includeTimestamps ? Date.now() : undefined,
        };
        process.stdout.write(JSON.stringify(jsonString));
      }
      logger.info(`received on topic "${topic}": \n${JSON.stringify(decoded)}`);
    } catch (error) {
      logger.error("Could not decode message:", { topic, message, error });
    }
  });
};

const cleanup = (exitCode) => {
  if (config.json.enabled && config.json.enclosingBrackets) {
    console.log("\n]");
  }
  logger.debug("...cleanup completed, exit code", exitCode);
  process.exit(exitCode || 0);
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

if (config.json.enabled) {
  const readline = require("readline");
  readline.emitKeypressEvents(process.stdin);
  process.stdin.setRawMode(true);
  process.stdin.on("keypress", (str, key) => {
    if (key.ctrl && key.name === "c") {
      cleanup();
    }
  });
}
process.on("SIGINT", cleanup);

main();
