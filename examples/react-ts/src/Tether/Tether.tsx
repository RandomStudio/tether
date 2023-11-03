import { useEffect, useState } from "react";
import { TetherAgent, IClientOptions, BROKER_DEFAULTS } from "tether-agent";

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
      ...BROKER_DEFAULTS.browser,
      host: props.host,
    };

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
      <h1>Tether @ {props.host}</h1>
      {isBusy ? (
        <div>Busy...</div>
      ) : (
        <div>
          <div>connected? {agent?.getIsConnected() ? "true" : "false"} : </div>
          <div>State: {agent?.getState()}</div>
        </div>
      )}
    </div>
  );
};
