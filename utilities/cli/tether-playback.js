#!/usr/bin/env node
// Third-party modules
const mqtt = require("async-mqtt");
const rc = require("rc");
const parse = require("parse-strings-in-object");
const { getLogger } = require("log4js");
const { parser } = require("stream-json");
const { chain } = require("stream-chain");
const { streamArray } = require("stream-json/streamers/StreamArray");
const { streamToRx } = require("rxjs-stream");
const { of } = require("rxjs");
const { concatMap, delay, finalize, tap } = require("rxjs/operators");

// Built-in modules
const fs = require("fs/promises");
const path = require("path");
const readline = require("readline");
const { resolve } = require("path");

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
    file: "./demo.json",
    loops: 1,
    loopInfinite: false,
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
      for (var i = 0; i < config.loops || config.loopInfinite; i++) {
        await startPlayback(client, filePath);
        logger.info("playback done");
      }
      logger.info("all loops completed");
      client.end();
      process.exit(0);
    } catch (e) {
      logger.fatal("could not connect to MQTT broker:", e);
    }
  } catch (e) {
    logger.fatal("Error reading file:", e);
    process.exit(1);
  }
};

const startPlayback = async (client, filePath) =>
  new Promise(async (resolve, reject) => {
    logger.debug("start playback, reading", filePath, "...");
    const fileHandle = await fs.open(filePath);
    const readStream = fileHandle.createReadStream();

    const pipeline = chain([
      readStream,
      parser(),
      streamArray(),
      (data) => data.value,
    ]);

    pipeline.on("end", () => {
      logger.debug("JSON stream pipeline ended!");
    });

    const messages$ = streamToRx(pipeline);

    const delayedMessages$ = messages$.pipe(
      concatMap((message) => of(message).pipe(delay(message.deltaTime))),
      tap((entry) => {
        client.publish(entry.topic, Buffer.from(entry.message.data));
        logger.trace("Send after", entry.deltaTime);
      }),
      finalize(() => {
        logger.debug("finalize!");
        resolve();
      })
    );

    delayedMessages$.subscribe();
  });

main();
