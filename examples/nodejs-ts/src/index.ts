import defaults from "./defaults";
import parse from "parse-strings-in-object";
import rc from "rc";
import { getLogger } from "log4js";
import { ChannelSender, TetherAgent } from "tether-agent";

const appName = defaults.appName;

const config: typeof defaults = parse(rc(appName, defaults));

const logger = getLogger(appName);
logger.level = config.loglevel;

logger.info("started with config", config);
logger.debug("Debug logging enabled; output could be verbose!");

const main = async () => {
  const agent = await TetherAgent.create("brain");

  const sender = new ChannelSender(agent, "randomValues");
  sender.encodeAndSend({
    value: Math.random(),
    timestamp: Date.now(),
    something: "one",
  });

  const typedSender = new ChannelSender<number>(
    agent,
    "randomValuesStrictlyTyped"
  );
  // This will be rejected by TypeScript compiler:
  // typedSender.encodeAndSend({
  //   value: Math.random(),
  //   timestamp: Date.now(),
  //   something: "one",
  // });

  // This will work fine, though
  typedSender.encodeAndSend(Math.random());
};

// ================================================
// Kick off main process here
main();
