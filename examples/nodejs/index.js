const {
  TetherAgent,
  ChannelInput,
  ChannelOutput,
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
  const outputChannel = new ChannelOutput(agent, "randomValue");
  const fastOutputChannel = new ChannelOutput(agent, "fastValues");
  const emptyOutputChannel = new ChannelOutput(agent, "emptyMessage");

  setInterval(() => {
    const m = {
      timestamp: Date.now(),
      value: Math.random(),
    };
    outputChannel.publish(Buffer.from(encode(m)));
  }, 1000);

  setInterval(() => {
    emptyOutputChannel.publish();
  }, 3333);

  let num = 0;
  setInterval(() => {
    num++;
    fastOutputChannel.publish(Buffer.from(encode(num)));
  }, 8);

  // Demonstrate Receiving
  const inputChannelOne = await ChannelInput.create(agent, "randomValue");
  inputChannelOne.on("message", (payload, topic) => {
    console.log("received:", { payload, topic });
    const m = decode(payload);
    console.log(">>>>>>>> received message on inputChannelOne:", { topic, m });
  });

  const inputEmptyMessages = await ChannelInput.create(agent, "emptyMessage");
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
