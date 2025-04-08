import { useEffect, useState } from "react";
import { ChannelReceiver, TetherAgent } from "tether-agent";

interface Props {
  agent: TetherAgent;
}

export const Receiver = (props: Props) => {
  const [channel, setChannel] = useState<ChannelReceiver | null>(null);
  const [lastMessage, setLastMessage] = useState("");

  useEffect(() => {
    ChannelReceiver.create(props.agent, "everything", {
      overrideTopic: "#",
    })
      .then((channel) => {
        setChannel(channel);
        channel.on("message", (payload, topic) => {
          console.log("Received message on", topic);
          const timestamp = Date.now();
          setLastMessage(JSON.stringify({ topic, payload, timestamp }));
        });
      })
      .catch((e) => {
        console.error("Error creating Channel Receiver:", e);
      });
  }, [props.agent]);

  return (
    <div>
      <h2>Channel Receiver</h2>
      <div>
        {channel && <code>{JSON.stringify(channel.getDefinition())}</code>}
      </div>
      <div>
        Last message: <code>{lastMessage}</code>
      </div>
    </div>
  );
};
