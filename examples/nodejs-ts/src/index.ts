import defaults from "./defaults";
import parse from "parse-strings-in-object";
import rc from "rc";
import { getLogger } from "log4js";
import { TetherAgent } from "tether-agent";

const appName = defaults.appName;

const config: typeof defaults = parse(rc(appName, defaults));

const logger = getLogger(appName);
logger.level = config.loglevel;

logger.info("started with config", config);
logger.debug("Debug logging enabled; output could be verbose!");

const main = async () => {
  const agent = await TetherAgent.create("testerNodeTS");

  // Note the alternative syntax for doing the same thing, below:
  // ...
  // const sender = new ChannelSender(agent, "randomValues");
  const sender = agent.getSender("randomValues");

  sender.send({
    value: Math.random(),
    timestamp: Date.now(),
    something: "one",
  });

  // This should result in the same channel being re-used!
  const sender2 = agent.getSender("randomValues");
  sender.send({
    value: Math.random(),
    timestamp: Date.now(),
    something: "two",
  });

  const genericReceiver = await agent.getReceiver("randomValuesStrictlyTyped");
  genericReceiver.on("message", (payload, topic) => {
    logger.info(
      "Our generic receiver got:",
      payload,
      typeof payload,
      "on topic",
      topic
    );
  });

  const typedReceiver = await agent.getReceiver<number>(
    "randomValuesStrictlyTyped"
  );
  typedReceiver.on("message", (payload) => {
    logger.info("Our typed receiver got", payload, typeof payload);
  });

  const reuseReceiver = await agent.getReceiver<number>(
    "randomValuesStrictlyTyped"
  );
  reuseReceiver.on("message", (payload) => {
    logger.info("Duplicate receiver also got:", payload, typeof payload);
  });

  const typedSender = agent.getSender<number>("randomValuesStrictlyTyped");
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
