const { TetherAgent } = require("@tether/tether-agent");
const parse = require("parse-strings-in-object");
const rc = require("rc");
const { encode, decode } = require("@msgpack/msgpack");

const agent = new TetherAgent("dummy", "NodeJSDummy");

const config = parse(rc("NodeJSDummy", {}));

console.log("Launch with config", config);

const main = async () => {
  await agent.connect();

  const output = agent.createOutput("randomValue");

  setInterval(() => {
    const m = {
      timestamp: Date.now(),
      value: Math.random(),
    };
    output.publish(Buffer.from(encode(m)));
  }, 1000);

  const input = agent.createInput("randomValue");
  input.on("message", (_topic, payload) => {
    console.log("received message:", payload);
    const m = decode(payload);
    console.log("decoded:", m);
  });
};

main();
