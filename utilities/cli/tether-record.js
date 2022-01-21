#!/usr/bin/env node
// Third-party modules
const mqtt = require("async-mqtt");
const rc = require("rc");
const parse = require("parse-strings-in-object");
const { getLogger } = require("log4js");

// Built-in modules
const fs = require("fs");
const path = require("path");

const appName = "tetherRecord";

const config = parse(
  rc(appName, {
    loglevel: "info",
    protocol: "tcp",
    host: "tether-io.dev",
    port: 1883,
    topic: "#",
    username: "tether",
    password: "sp_ceB0ss!",
    path: "",
    file: {
      basePath: "./",
      baseName: "recording",
      nameIncludesTimestamp: true,
    },
  })
);

const logger = getLogger(appName);
logger.level = config.loglevel;

logger.debug(appName, "launched with config", JSON.stringify(config, null, 2));

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
  const { basePath, baseName, nameIncludesTimestamp } = config.file;
  const filePath =
    path.resolve(basePath, baseName) +
    (nameIncludesTimestamp ? `_${Date.now()}` : "") +
    ".json";
  logger.info("Will write to", filePath);
  logger.debug("Subscribing to client with topic", config.topic, "...");
  client.subscribe(config.topic);

  fs.writeFileSync(filePath, "[\n");

  const startTime = Date.now();
  let count = 0;

  client.on("message", (topic, message) => {
    count++;
    const entry = {
      topic,
      message,
      deltaTime: Date.now() - startTime,
    };
    logger.debug("Writing", entry);
    fs.appendFileSync(
      filePath,
      (count > 1 ? ",\n" : "") + JSON.stringify(entry, null, 0)
    );
  });

  process.on("SIGINT", () => {
    logger.info("Interrupt called; close file first...");
    fs.appendFileSync(filePath, "\n]");
    logger.info("Quit now");
    process.exit(0);
  });
};

main();
