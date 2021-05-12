import MsgPack from "msgpack5";
import TetherAgent from "./base_agent";

import defaults from "./defaults";
import parse from "parse-strings-in-object";
import rc from "rc";
import { getLogger } from "log4js";

const config: typeof defaults = parse(rc("tether2", defaults));

const logger = getLogger();
logger.level = config.loglevel;

logger.info("started with config", config);

const agent = new TetherAgent("dummy", "nodejs_dummy");

(async () => {
  const sender = await agent.createOutput("DummyData");
  console.log("got sender!");

  setInterval(() => {
    const msg = "yo from nodeJS";
    // console.log("sending");
    sender.publish(Buffer.from(msg));
  }, 3000);
})();

// const msgpack = MsgPack();

// const encoded = msgpack.encode({   hello: "world" });

// console.log({ number: encoded, str: encoded.toString("hex") });
// // 81a568656c6c6fa5776f726c64s

// const incoming = Buffer.from([
//   0x81,
//   0xa5,
//   0x68,
//   0x65,
//   0x6c,
//   0x6c,
//   0x6f,
//   0xa5,
//   0x77,
//   0x6f,
//   0x72,
//   0x6c,
//   0x64,
// ]);

// console.log({
//   actual: msgpack.decode(encoded),
//   shouldMatch: msgpack.decode(incoming),
// });
