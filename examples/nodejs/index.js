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
console.log("Launch with config", config);

const main = async () => {
  const agent = await TetherAgent.create(
    "dummy",
    config.clientOptions,
    config.loglevel,
    "NodeJSDummy"
  );
  // setTimeout(() => {
  //   agent.connect(config.clientOptions);
  // }, 5000);
  const outputPlug = agent.createOutput("randomValue");
  const emptyOutputPlug = agent.createOutput("emptyMessage");

  setInterval(() => {
    const m = {
      timestamp: Date.now(),
      value: Math.random(),
    };
    outputPlug.publish(Buffer.from(encode(m)));

    emptyOutputPlug.publish();
  }, 1000);

  const inputPlugOne = agent.createInput("randomValue");
  inputPlugOne.onMessage((payload, topic) => {
    console.log("received:", { payload, topic });
    const m = decode(payload);
    console.log("received message on inputPlugOne:", { topic, m });
  });

  const inputPlugTwo = agent.createInput(
    "moreRandomValues",
    "dummy/NodeJSDummy/randomValue"
  );
  inputPlugTwo.onMessage((payload, topic) => {
    const m = decode(payload);
    console.log("received message on inputPlugTwo:", { topic, m });
  });

  const inputPlugThree = agent.createInput(
    "evenMoreRandomValues",
    "+/+/randomValue"
  );
  inputPlugThree.onMessage((payload, topic) => {
    const m = decode(payload);
    console.log("received message on inputPlugThree:", { topic, m });
  });

  try {
    const inputPlugFour = agent.createInput("randomValue", "+/+/somethingElse");
    inputPlugFour.onMessage(() => {
      throw Error(
        "we didn't expect to receive anything on this plug, despite the name"
      );
    });
  } catch (e) {
    console.log("we expected an error here; duplicate plug names!");
  }

  let countReceived = 0;
  const inputPlugJustOnce = agent.createInput(
    "randomValueOnce",
    "+/+/randomValue"
  );
  inputPlugJustOnce.onMessageOnce((payload, topic) => {
    countReceived++;
    console.log("received", countReceived, "message on inputPlugJustOnce");
    if (countReceived > 1) {
      throw Error("we should only be able to receive one message on this plug");
    }
  });

  const inputEmptyMessages = agent.createInput("emptyMessage");
  inputEmptyMessages.onMessage((payload, topic) => {
    console.log("received empty message:", { payload, topic });
  });
};

main();
