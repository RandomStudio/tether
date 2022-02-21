const { TetherAgent } = require("@tether/tether-agent");
const parse = require("parse-strings-in-object");
const rc = require("rc");
const { encode, decode } = require("@msgpack/msgpack");

const agent = new TetherAgent("dummy", "NodeJSDummy", "trace");

const config = parse(rc("NodeJSDummy", {}));

console.log("Launch with config", config);

const main = async () => {
  await agent.connect();

  const outputPlug = agent.createOutput("randomValue");

  setInterval(() => {
    const m = {
      timestamp: Date.now(),
      value: Math.random(),
    };
    outputPlug.publish(Buffer.from(encode(m)));
  }, 1000);

  const inputPlugOne = agent.createInput("randomValue");
  inputPlugOne.on("message", (topic, payload) => {
    const m = decode(payload);
    console.log("received message on inputPlugOne:", { topic, m });
  });

  const inputPlugTwo = agent.createInput(
    "moreRandomValues",
    "dummy/NodeJSDummy/randomValue"
  );
  inputPlugTwo.on("message", (topic, payload) => {
    const m = decode(payload);
    console.log("received message on inputPlugTwo:", { topic, m });
  });

  const inputPlugThree = agent.createInput(
    "moreRandomValues",
    "+/+/randomValue"
  );
  inputPlugThree.on("message", (topic, payload) => {
    const m = decode(payload);
    console.log("received message on inputPlugThree:", { topic, m });
  });
};

main();
