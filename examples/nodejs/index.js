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

  const input = await agent.createInput("browserData");
  input.on("message", (topic, message) => {
    const decoded = msgpack.decode(message);
    console.log("received message on plug level:", { topic, message, decoded });
  });

  const [sender1, sender2] = await Promise.all([
    agent.createOutput("dummyData"),
    agent.createOutput("someOtherData"),
  ]);
  console.log("got senders!", {
    sender1: sender1.getDefinition(),
    sender2: sender2.getDefinition(),
  });

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
    const encoded1 = msgpack.encode(msg);
    i++;

    sender1.publish(Buffer.from(encoded1));

    const encoded2 = msgpack.encode({ hello: "boo!" });
    sender2.publish(Buffer.from(encoded2));
  }, 3000);
})();
