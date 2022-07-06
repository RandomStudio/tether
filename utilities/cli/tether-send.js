#!/usr/bin/env node
const mqtt = require("async-mqtt");
const rc = require("rc");
const parse = require("parse-strings-in-object");
const { encode } = require("@msgpack/msgpack");
const { getLogger } = require("log4js");
const path = require("path");
const fs = require("fs/promises");

const config = parse(
  rc("tetherSend", {
    loglevel: "info",
    protocol: "tcp",
    host: "tether-io.dev",
    port: 1883,
    topic: `tether-send/unknown/dummy`,
    message: '{ "hello": "world" }',
    username: "tether",
    password: "sp_ceB0ss!",
    path: "",
    jsonReader: {
      enabled: false,
      path: "./data.json",
    },
  })
);

const logger = getLogger("tether-send");
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

const sendFromJson = async (client, filePath) => {
  const res = await fs.readFile(filePath);
  logger.trace({ res });
  const json = JSON.parse(res);
  logger.trace({ json });
  logger.info("Read array from JSON of length", json.length);

  let previousTime = json[0].timestamp;

  await Promise.all(
    json.map(
      (entry, index) =>
        new Promise((resolve, reject) => {
          const { topic, timestamp, ...rest } = entry;
          const delay = timestamp - previousTime;
          setTimeout(() => {
            const encoded = encode(rest);
            logger.debug("send", { rest, delay });
            client.publish(topic, Buffer.from(encoded));
            resolve();
          }, delay);
        })
    )
  );
};

const main = async () => {
  const { protocol, host, port, username, password } = config;

  const url = `${protocol}://${host}:${port}${config.path}`;

  logger.info("Connecting to MQTT broker @", url, "...");

  try {
    const client = await mqtt.connectAsync(url, { username, password });
    logger.info("...connected OK");

    if (config.jsonReader.enabled) {
      const filePath = path.resolve(config.jsonReader.path);
      logger.info("jsonReader enabled; will read from file", filePath, "...");
      await sendFromJson(client, filePath);
    } else {
      logger.debug("jsonReader not enabled; will send single message");
      sendMessages(client, config.message, config.topic);
    }

    // TODO: should loop / REPL before closing
    client.end();
  } catch (e) {
    logger.fatal("could not connect to MQTT broker:", e);
  }
};

main();
