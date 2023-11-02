const {
  TetherAgent,
  InputPlug,
  OutputPlug,
  BROKER_DEFAULTS,
  encode,
  decode,
} = require("tether-agent");
const parse = require("parse-strings-in-object");
const rc = require("rc");

const config = parse(
  rc("NodeJSDummy", {
    loglevel: "debug",
    clientOptions: {},
  })
);
console.log("Launch with config", config);

const main = async () => {
  const agent = await TetherAgent.create("dummy", {
    loglevel: config.loglevel,
  });

  // Example of custom options below:
  // const agent = await TetherAgent.create("dummy", {
  //   id: "special",
  //   brokerOptions: BROKER_DEFAULTS.browser,
  // });

  const outputPlug = new OutputPlug(agent, "randomValue");
  const emptyOutputPlug = new OutputPlug(agent, "emptyMessage");

  setInterval(() => {
    const m = {
      timestamp: Date.now(),
      value: Math.random(),
    };
    outputPlug.publish(Buffer.from(encode(m)));

    emptyOutputPlug.publish();
  }, 1000);

  setTimeout(() => {
    const fastOutput = new OutputPlug(agent, "fastValues", undefined, {
      qos: 0,
      retain: false,
    });
    setInterval(() => {
      const a = [Math.random(), Math.random(), Math.random()];
      fastOutput.publish(Buffer.from(encode(a)));
    }, 10);
  }, 4000);

  const fastInput = new InputPlug(
    agent,
    "fastValuesReceiver",
    "+/+/fastValues"
  );
  fastInput.on("message", (payload) => {
    console.log("received fastValues");
  });

  const inputPlugOne = new InputPlug(agent, "randomValue");
  inputPlugOne.on("message", (payload, topic) => {
    console.log("received:", { payload, topic });
    const m = decode(payload);
    console.log("received message on inputPlugOne:", { topic, m });
  });

  const inputPlugTwo = new InputPlug(
    agent,
    "moreRandomValues",
    "dummy/NodeJSDummy/randomValue"
  );
  inputPlugTwo.on("message", (payload, topic) => {
    const m = decode(payload);
    console.log("received message on inputPlugTwo:", { topic, m });
  });

  const inputPlugThree = new InputPlug(
    agent,
    "evenMoreRandomValues",
    "+/+/randomValue"
  );
  inputPlugThree.on("message", (payload, topic) => {
    const m = decode(payload);
    console.log("received message on inputPlugThree:", { topic, m });
  });

  try {
    const inputPlugFour = new InputPlug(
      agent,
      "randomValue",
      "+/+/somethingElse"
    );
    inputPlugFour.on("message", () => {
      throw Error(
        "we didn't expect to receive anything on this plug, despite the name"
      );
    });
  } catch (e) {
    console.log("we expected an error here; duplicate plug names!");
  }

  let countReceived = 0;
  const inputPlugJustOnce = new InputPlug(
    agent,
    "randomValueOnce",
    "+/+/randomValue"
  );
  inputPlugJustOnce.once("message", (payload, topic) => {
    countReceived++;
    console.log("received", countReceived, "message on inputPlugJustOnce");
    if (countReceived > 1) {
      throw Error("we should only be able to receive one message on this plug");
    }
  });

  const inputEmptyMessages = new InputPlug(agent, "emptyMessage");
  inputEmptyMessages.on("message", (payload, topic) => {
    console.log("received empty message:", { payload, topic });
  });
};

main()
  .then(() => {
    // done
  })
  .catch((e) => {
    console.error("Error in main function: ", e);
  });
