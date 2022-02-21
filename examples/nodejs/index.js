const { TetherAgent } = require("@tether/tether-agent");
const parse = require("parse-strings-in-object");
const rc = require("rc");
const agent = new TetherAgent("dummy", "NodeJSDummy");

const config = parse(rc("NodeJSDummy", {}));

console.log("Launch with config", config);

const main = async () => {
  await agent.connect();
};

main();
