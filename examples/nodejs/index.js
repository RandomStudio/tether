const {
  TetherAgent,
  InputPlug,
  OutputPlug,
  NODEJS,
  encode,
  decode,
} = require("tether-agent");
const parse = require("parse-strings-in-object");
const rc = require("rc");

const config = parse(
  rc("NodeJSDummy", {
    loglevel: "debug",
    host: "localhost",
    port: 1883,
    protocol: "tcp",
  })
);
console.log("Launch with config", config);

const main = async () => {
  const agent = await TetherAgent.create("dummy", {
    loglevel: config.loglevel,
    brokerOptions: {
      ...NODEJS,
      host: config.host,
      port: config.port,
      protocol: config.protocol,
    },
  });

  // Demonstrate Publishing
  const outputPlug = new OutputPlug(agent, "randomValue");
  const emptyOutputPlug = new OutputPlug(agent, "emptyMessage");

  setInterval(() => {
    const m = {
      timestamp: Date.now(),
      value: Math.random(),
    };
    outputPlug.publish(Buffer.from(encode(m)));
  }, 1000);

  setInterval(() => {
    emptyOutputPlug.publish();
  }, 3333);

  // Demonstrate Receiving
  const inputPlugOne = await InputPlug.create(agent, "randomValue");
  inputPlugOne.on("message", (payload, topic) => {
    console.log("received:", { payload, topic });
    const m = decode(payload);
    console.log(">>>>>>>> received message on inputPlugOne:", { topic, m });
  });

  const inputEmptyMessages = await InputPlug.create(agent, "emptyMessage");
  inputEmptyMessages.on("message", (payload, topic) => {
    console.log(">>>>>>>> received empty message:", { payload, topic });
  });
};

main()
  .then(() => {
    // done
  })
  .catch((e) => {
    console.error("Error in main function: ", e);
  });
