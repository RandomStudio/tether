#!/usr/bin/env node
// Third-party modules
const mqtt = require("async-mqtt");
const rc = require("rc");
const parse = require("parse-strings-in-object");
const { getLogger } = require("log4js");
const { chain } = require("stream-chain");
const { parser } = require("stream-json");
const { streamArray } = require("stream-json/streamers/StreamArray");
const { fromEvent, of } = require("rxjs");
const {
  concatMap,
  delay,
  filter,
  finalize,
  map,
  takeUntil,
  scan,
  tap,
  withLatestFrom,
  repeat,
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

  const messages$ = fromEvent(pipeline, "data").pipe(
    map((tokenizedJson) => {
      logger.trace({ tokenizedJson });
      return tokenizedJson.value;
    })
  );

  const totalCount$ = messages$.pipe(scan((acc, _) => acc + 1, 0));

  const timedMessages$ = messages$.pipe(
    // delay emit messages by delta
    concatMap((message) => of(message).pipe(delay(message.deltaTime))),
    // then send with simulated topic
    tap((entry) => {
      logger.trace("Send after", entry.deltaTime);
      logger.debug("Sending", entry);
      client.publish(entry.topic, Buffer.from(entry.message.data));
      // logger.warn("TODO: send the message now!");
    }),
    scan((acc, _) => acc + 1, 0),
    // tap((count) => logger.debug("messages sent:", count)),
    withLatestFrom(totalCount$),
    tap((both) => logger.debug("both:", both))
  );

  const compareCounts$ = timedMessages$.pipe(
    tap((val) => logger.debug("compareCounts:", val)),
    filter(([soFar, total]) => soFar === total)
  );

  compareCounts$.subscribe((x) => logger.info("compare", x));

  const reachedEnd$ = timedMessages$.pipe(
    takeUntil(compareCounts$),
    finalize(() => {
      logger.info("all done");
    })
  );

  reachedEnd$.subscribe();
};
main();
