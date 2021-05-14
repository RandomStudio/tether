const MsgPack = require("msgpack5");
const { TetherAgent } = require("tether");

console.log(TetherAgent);
const agent = new TetherAgent("dummy", "nodejs_dummy");
const msgpack = MsgPack();

(async () => {
  try {
    await agent.connect();
  } catch (e) {
    console.error("failed to connect:", e);
    process.exit(1);
  }

  const input = await agent.createInput("BrowserData");
  input.on("message", (topic, message) => {
    const decoded = msgpack.decode(message);
    console.log("received message:", { topic, message, decoded });
  });

  const sender = await agent.createOutput("DummyData");
  console.log("got sender!");

  let i = 0;

  setInterval(() => {
    const randomArray = [Math.random(), Math.random(), Math.random()];
    const msg = {
      from: "nodeJS",
      hello: "world",
      someNumber: i,
      isEven: i % 2 === 0,
      randomArray,
    };
    const encoded = msgpack.encode(msg);
    i++;
    console.log("sending", {
      msg,
      encoded,
      mType: typeof encoded,
      size: encoded.length,
    });

    sender.publish(Buffer.from(encoded));
  }, 3000);
})();
