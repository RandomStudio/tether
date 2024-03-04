import { useEffect, useState } from "react";
import { InputPlug, TetherAgent } from "tether-agent";

interface Props {
  agent: TetherAgent;
}

export const Receiver = (props: Props) => {
  const [plug, setPlug] = useState<InputPlug | null>(null);
  const [lastMessage, setLastMessage] = useState("");

  useEffect(() => {
    InputPlug.create(props.agent, "everything", {
      overrideTopic: "#",
    })
      .then((plug) => {
        setPlug(plug);
        plug.on("message", (payload, topic) => {
          console.log("Received message on", topic);
          const timestamp = Date.now();
          setLastMessage(JSON.stringify({ topic, payload, timestamp }));
        });
      })
      .catch((e) => {
        console.error("Error creating InputPlug:", e);
      });
  }, [props.agent]);

  return (
    <div>
      <h2>Input Plug</h2>
      <div>{plug && <code>{JSON.stringify(plug.getDefinition())}</code>}</div>
      <div>
        Last message: <code>{lastMessage}</code>
      </div>
    </div>
  );
};
