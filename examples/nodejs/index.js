const { encode, decode } = require("@msgpack/msgpack");
const { TetherAgent } = require("tether-agent");

// console.log(TetherAgent);
const agent = new TetherAgent("dummy", "nodejs_dummy");

const LogType = Object.freeze({
  Log: "log",
  Info: "info",
  Warn: "warn",
  Error: "error",
});

const log = (type, ...content) => {
  if (!Object.keys(LogType).includes(type)) {
    type = LogType.Log;
  }
  console[type](`[${new Date().toLocaleString()}]`, ...content);
};

const MQTTConnect = async () => {
  log(LogType.Info, "Connecting to MQTT");
  try {
    await agent.connect();
  } catch (e) {
    log(LogType.Info, "Retrying connection in 5 seconds.");
    setTimeout(MQTTConnect, 5000);
    return;
  }

  const inputBrowser = await agent.createInput("browserData");
  inputBrowser.on("message", (topic, message) => {
    const decoded = decode(message);
    log(LogType.Info, "Received message on plug level:", {
      topic,
      message,
      decoded,
    });
  });

  const inputMCU = await agent.createInput("mcuData");
  inputMCU.on("message", (topic, message) => {
    log(LogType.Info, "Received message on mcuData");
    try {
      const decoded = decode(message);
      log(LogType.Info, "Received message on plug level:", {
        topic,
        message,
        decoded,
      });
    } catch (e) {
      console.error("error decoding:", { e, message });
    }
  });

  const [sender1, sender2] = await Promise.all([
    agent.createOutput("dummyData"),
    agent.createOutput("someOtherData"),
  ]);
  log(LogType.Info, "got senders!", {
    sender1: sender1.getDefinition(),
    sender2: sender2.getDefinition(),
  });

  let i = 0;

  let remaining = 10;

  setInterval(() => {
    const randomArray = [Math.random(), Math.random(), Math.random()];
    const msg = {
      from: "nodeJS",
      hello: "world",
      someNumber: i,
      isEven: i % 2 === 0,
      randomArray,
    };
    const encoded1 = encode(msg);
    i++;

    sender1.publish(Buffer.from(encoded1));
    log(
      LogType.Info,
      `Sent message on topic ${sender1.getDefinition().topic}:`,
      msg,
      Buffer.from(encoded1)
    );

    const msg2 = { hello: "boo!" };
    const encoded2 = encode(msg2);
    sender2.publish(Buffer.from(encoded2));

    remaining--;
    if (remaining <= 0) {
      console.log("Done! Closing and quitting...");
      agent.disconnect().then(() => {
        console.log("disconnected OK");
        process.exit(0);
      });
    }
    log(
      LogType.Info,
      `Sent message on topic ${sender2.getDefinition().topic}:`,
      msg2
    );
  }, 3000);
};

MQTTConnect();
