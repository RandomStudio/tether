import { useEffect, useState } from "react";
import { ChannelSender, TetherAgent } from "tether-agent";

interface Props {
  agent: TetherAgent;
}

export const Sender = (props: Props) => {
  const { agent } = props;

  useEffect(() => {
    console.log("Sender useEffect");
    setChannel(new ChannelSender(agent, "sender"));
  }, [agent]);

  const [useCustomTopic, setUseCustomTopic] = useState(false);
  const [customTopic, setTCustomTopic] = useState("");
  const [channel, setChannel] = useState<ChannelSender<unknown> | null>(null);

  return (
    <div>
      <h2>Channel Sender</h2>
      <div>
        {channel && <code>{JSON.stringify(channel.getDefinition())}</code>}
      </div>
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
                setChannel(
                  agent.createSender("sender", {
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
                setChannel(agent.createSender("sender"));
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
      {channel && (
        <div>
          <button
            onClick={async () => {
              try {
                await channel.send();
              } catch (e) {
                console.error("We got an error when trying to publish:", e);
                console.log("agent connected?", agent.getIsConnected());
                console.log("agent state?", agent.getState());
                console.log("agent client?", agent.getClient());
              }
            }}
          >
            Send (empty)
          </button>
        </div>
      )}
    </div>
  );
};
