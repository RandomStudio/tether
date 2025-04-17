import defaults from "./defaults";
import parse from "parse-strings-in-object";
import rc from "rc";
import { getLogger } from "log4js";
import { ChannelReceiver, ChannelSender, TetherAgent } from "tether-agent";

const appName = defaults.appName;

const config: typeof defaults = parse(rc(appName, defaults));

const logger = getLogger(appName);
logger.level = config.loglevel;

logger.info("started with config", config);
logger.debug("Debug logging enabled; output could be verbose!");

const main = async () => {
  const agent = await TetherAgent.create("brain");

  // Note the alternative syntax for doing the same thing, below:
  // ...
  // const sender = new ChannelSender(agent, "randomValues");
  const sender = agent.createSender("randomValues");

  sender.send({
    value: Math.random(),
    timestamp: Date.now(),
    something: "one",
  });

  const genericReceiver = await ChannelReceiver.create(
    agent,
    "randomValuesStrictlyTyped"
  );
  genericReceiver.on("message", (payload, topic) => {
    logger.info(
      "Our generic receiver got:",
      payload,
      typeof payload,
      "on topic",
      topic
    );
  });

  const typedReceiver = await ChannelReceiver.create<number>(
    agent,
    "randomValuesStrictlyTyped"
  );
  typedReceiver.on("message", (payload) => {
    logger.info("Our typed receiver got", payload, typeof payload);
  });

  const typedSender = new ChannelSender<number>(
    agent,
    "randomValuesStrictlyTyped"
  );
  // This will be rejected by TypeScript compiler:
  // typedSender.send({
  //   value: Math.random(),
  //   timestamp: Date.now(),
  //   something: "one",
  // });

  // This will work fine, though
  typedSender.send(Math.random());

  setTimeout(() => {
    agent.disconnect();
  }, 2000);
};

// ================================================
// Kick off main process here
main();
