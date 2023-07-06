#!/usr/bin/env node
const mqtt = require("async-mqtt");
const rc = require("rc");
const parse = require("parse-strings-in-object");
const { getLogger } = require("log4js");

let topics = [];
let agentTypes = [];
let agentIds = [];
let outputNames = [];

const config = parse(
  rc("tetherTopics", {
    loglevel: "info",
    protocol: "tcp",
    host: "localhost",
    port: 1883,
    username: "tether",
    password: "sp_ceB0ss!",
    path: "",
  })
);

const logger = getLogger("tether-topics");
logger.level = config.loglevel;

logger.debug(
  "tether-topics CLI launched with config",
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
    const updateTopics = appendToListIfNew(topics, topic);
    if (updateTopics.length !== topics.length) {
      topics = updateTopics;
      try {
        const { agentType, agentId, outputName } = getTopicElements(topic);
        agentTypes = appendToListIfNew(agentTypes, agentType);
        agentIds = appendToListIfNew(agentIds, agentId);
        outputNames = appendToListIfNew(outputNames, outputName);
        logger.info({ topics, agentTypes, agentIds, outputNames });
      } catch (e) {
        logger.error("Error getting topic elements:", e);
      }
    }
  });
};

const getTopicElements = (topic) => {
  const parts = topic.split("/");
  if (parts.length !== 3) {
    throw Error(
      "not a valid Tether topic routing key:" + JSON.stringify({ topic, parts })
    );
  }
  const [agentType, agentId, outputName] = parts;
  return { agentType, agentId, outputName };
};

const appendToListIfNew = (list, item) => {
  const match = list.includes(item);
  if (!match) {
    logger.debug("append", item);
    return [...list, item];
  } else {
    return list;
  }
};

main();
