import { TetherAgent, ChannelReceiver, ChannelSender } from "tether-agent";

const main = async () => {
  const agent = await TetherAgent.create("dummy");

  console.log("Agent instance", agent.getConfig(), agent.getIsConnected());

  // Demonstrate Sending
  const channelSender = new ChannelSender(agent, "randomValue");
  const fastChannelSender = new ChannelSender(agent, "fastValues", {
    publishOptions: { qos: 0 },
  });
  const emptyChannelSender = new ChannelSender(agent, "emptyMessage");

  let quitting = false;

  setInterval(() => {
    const m = {
      timestamp: Date.now(),
      value: Math.random(),
    };
    if (!quitting) {
      channelSender.send(m);
    }
  }, 1000);

  setInterval(() => {
    if (!quitting) {
      emptyChannelSender.send();
    }
  }, 3333);

  let num = 0;
  setInterval(() => {
    num++;
    if (!quitting) {
      fastChannelSender.send(num);
    }
  }, 8);

  setTimeout(() => {
    console.log("Ready to stop!");
    quitting = true;
    agent
      .disconnect()
      .then(() => {
        process.exit(0);
      })
      .catch((e) => {
        console.error("Error disconnecting:", e);
      });
  }, 5000);

  // Demonstrate Receiving
  const inputChannelOne = await ChannelReceiver.create(agent, "randomValue");
  inputChannelOne.on("message", (payload, topic) => {
    console.log("received:", { payload, topic });
    const m = payload;
    console.log(">>>>>>>> received message on inputChannelOne:", { topic, m });
  });

  const inputEmptyMessages = await ChannelReceiver.create(
    agent,
    "emptyMessage"
  );
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
