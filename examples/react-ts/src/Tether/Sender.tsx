import { useEffect, useState } from "react";
import { InputPlug, OutputPlug, TetherAgent } from "tether-agent";

interface Props {
  agent: TetherAgent;
}

export const Sender = (props: Props) => {
  useEffect(() => {
    setPlug(new OutputPlug(props.agent, "sender"));
  }, [props.agent]);

  const [useCustomTopic, setUseCustomTopic] = useState(false);
  const [customTopic, setTCustomTopic] = useState("");
  const [plug, setPlug] = useState<OutputPlug | null>(null);

  return (
    <div>
      <h2>Output Plug</h2>
      <div>{plug && <code>{JSON.stringify(plug.getDefinition())}</code>}</div>
      <div>
        {useCustomTopic ? (
          <>
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
            <button
              onClick={() => {
                setUseCustomTopic(false);
                setPlug(new OutputPlug(props.agent, "sender"));
              }}
            >
              Back to default
            </button>
          </>
        ) : (
          <>
            <button onClick={() => setUseCustomTopic(true)}>
              Use custom topic
            </button>
          </>
        )}
      </div>
      {plug && (
        <div>
          <button onClick={() => plug.publish()}>Send (empty)</button>
        </div>
      )}
    </div>
  );
};
