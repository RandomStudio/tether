const {
  TetherAgent,
  ChannelReceiver,
  ChannelSender,
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

  console.log("Agent instance", agent.getConfig(), agent.getIsConnected());

  // Demonstrate Publishing
  const channelSender = new ChannelSender(agent, "randomValue");
  const fastChannelSender = new ChannelSender(agent, "fastValues");
  const emptyChannelSender = new ChannelSender(agent, "emptyMessage");

  setInterval(() => {
    const m = {
      timestamp: Date.now(),
      value: Math.random(),
    };
    channelSender.send(Buffer.from(encode(m)));
  }, 1000);

  setInterval(() => {
    emptyChannelSender.send();
  }, 3333);

  let num = 0;
  setInterval(() => {
    num++;
    fastChannelSender.send(Buffer.from(encode(num)));
  }, 8);

  // Demonstrate Receiving
  const inputChannelOne = await ChannelReceiver.create(agent, "randomValue");
  inputChannelOne.on("message", (payload, topic) => {
    console.log("received:", { payload, topic });
    const m = decode(payload);
    console.log(">>>>>>>> received message on inputChannelOne:", { topic, m });
  });

  const inputEmptyMessages = await ChannelReceiver.create(
    agent,
    "emptyMessage"
  );
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
