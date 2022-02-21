const { TetherAgent } = require("@tether/tether-agent");
const parse = require("parse-strings-in-object");
const rc = require("rc");
const { encode, decode } = require("@msgpack/msgpack");

const config = parse(
  rc("NodeJSDummy", {
    loglevel: "debug",
    clientOptions: {},
  })
);
const agent = new TetherAgent("dummy", "NodeJSDummy", config.loglevel);

console.log("Launch with config", config);

const main = async () => {
  setTimeout(() => {
    agent.connect(config.clientOptions);
  }, 5000);
  // await agent.connect(config.clientOptions);

  const outputPlug = agent.createOutput("randomValue");

  setInterval(() => {
    const m = {
      timestamp: Date.now(),
      value: Math.random(),
    };
    outputPlug.publish(Buffer.from(encode(m)));
  }, 1000);

  const inputPlugOne = agent.createInput("randomValue");
  inputPlugOne.on("message", (payload, topic) => {
    console.log("received:", { payload, topic });
    const m = decode(payload);
    console.log("received message on inputPlugOne:", { topic, m });
  });

  const inputPlugTwo = agent.createInput(
    "moreRandomValues",
    "dummy/NodeJSDummy/randomValue"
  );
  inputPlugTwo.on("message", (payload, topic) => {
    const m = decode(payload);
    console.log("received message on inputPlugTwo:", { topic, m });
  });

  const inputPlugThree = agent.createInput(
    "evenMoreRandomValues",
    "+/+/randomValue"
  );
  inputPlugThree.on("message", (payload, topic) => {
    const m = decode(payload);
    console.log("received message on inputPlugThree:", { topic, m });
  });

  const inputPlugFour = agent.createInput("randomValue", "+/+/somethingElse");
  inputPlugFour.on("message", () => {
    throw Error(
      "we didn't expect to receive anything on this plug, despite the name"
    );
  });
};

main();
