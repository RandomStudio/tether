#!/usr/bin/env node
// Third-party modules
const mqtt = require("async-mqtt");
const rc = require("rc");
const parse = require("parse-strings-in-object");
const { getLogger } = require("log4js");
const { chain } = require("stream-chain");
const { parser } = require("stream-json");
const { pick } = require("stream-json/filters/Pick");
const { streamArray } = require("stream-json/streamers/StreamArray");
const { fromEvent, Observable, of } = require("rxjs");
const {
  concatMap,
  delay,
  endWith,
  filter,
  finalize,
  find,
  map,
  pairwise,
  startWith,
  takeUntil,
  tap,
  withLatestFrom,
} = require("rxjs/operators");

// Built-in modules
const fs = require("fs/promises");
const path = require("path");
const readline = require("readline");

const appName = "tetherPlayback";

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
    file: "./recording.json",
  })
);

const logger = getLogger(appName);
logger.level = config.loglevel;

logger.debug(appName, "launched with config", JSON.stringify(config, null, 2));

const main = async () => {
  const filePath = path.resolve(config.file);
  logger.info("Will read from file", filePath, "...");
  try {
    await fs.stat(filePath);
    logger.debug("Read file OK");

    const { protocol, host, port, username, password, path } = config;
    const url = `${protocol}://${host}:${port}${path}`;
    logger.info("Connecting to MQTT broker @", url, "...");

    try {
      const client = await mqtt.connectAsync(url, { username, password });
      logger.info("...connected OK");
      startPlayback(client, filePath);
    } catch (e) {
      logger.fatal("could not connect to MQTT broker:", e);
    }
  } catch (e) {
    logger.fatal("Error reading file:", e);
    process.exit(1);
  }
};

const startPlayback = async (client, filePath) => {
  logger.debug("start playback, reading", filePath, "...");
  const fileHandle = await fs.open(filePath);
  const readStream = fileHandle.createReadStream();

  const pipeline = chain([readStream, parser(), streamArray()]);
  pipeline.on("data", (d) => {
    logger.trace("entry", d.value);
  });

  const messages$ = fromEvent(pipeline, "data").pipe(
    map((tokenizedJson) => {
      logger.debug({ tokenizedJson });
      return tokenizedJson.value;
    })
  );

  messages$.subscribe((el) => logger.debug("element:", el));
};
main();
