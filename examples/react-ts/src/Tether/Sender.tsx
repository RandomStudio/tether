import { useState } from "react";
import { InputPlug, OutputPlug, TetherAgent } from "tether-agent";

interface Props {
  agent: TetherAgent;
}

export const Sender = (props: Props) => {
  const [customTopic, setTCustomTopic] = useState("");
  const [plug, setPlug] = useState<OutputPlug | null>(null);

  return (
    <div>
      <div>
        <input
          type="text"
          value={customTopic}
          onChange={(e) => setTCustomTopic(e.target.value)}
        ></input>
        <button
          onClick={() =>
            setPlug(
              new OutputPlug(props.agent, "sender", {
                overrideTopic: customTopic,
              })
            )
          }
        >
          Set topic
        </button>
      </div>
      {plug && (
        <div>
          <button onClick={() => plug.publish()}>Send (empty)</button>
        </div>
      )}
    </div>
  );
};
