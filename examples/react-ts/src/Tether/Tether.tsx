import { useEffect, useState } from "react";
import { TetherAgent, IClientOptions, BROWSER } from "tether-agent";
import { Sender } from "./Sender";
import { Receiver } from "./Receiver";

interface Props {
  host: string;
}

export const Tether = (props: Props) => {
  const [isBusy, setIsBusy] = useState(false);
  const [agent, setAgent] = useState<TetherAgent>();

  useEffect(() => {
    setIsBusy(true);
    console.log("New Tether Agent with host", props.host);

    const brokerOptions: IClientOptions = {
      ...BROWSER,
      host: props.host,
    };

    console.log("Connecting...");
    TetherAgent.create("browserDemo", { brokerOptions })
      .then((agent) => {
        setAgent(agent);
        console.info("Tether connect OK");
        setIsBusy(false);
      })
      .catch((e) => {
        console.error("Error init Tether:", e);
        setIsBusy(false);
      });
  }, [props.host]);

  return (
    <div>
      <h1>Tether Agent ↔︎ {props.host}</h1>
      {isBusy ? (
        <div>Busy...</div>
      ) : (
        <div>
          <div>connected? {agent?.getIsConnected() ? "true" : "false"} : </div>
          <div>State: {agent?.getState()}</div>
          {agent && (
            <>
              <Sender agent={agent} />
              <Receiver agent={agent} />
            </>
          )}
        </div>
      )}
    </div>
  );
};
