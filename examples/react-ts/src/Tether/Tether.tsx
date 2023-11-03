import { useEffect, useState } from "react";
import { TetherAgent, IClientOptions, BROKER_DEFAULTS } from "tether-agent";

interface Props {
  host: string;
}

export const Tether = (props: Props) => {
  const [agent, setAgent] = useState<TetherAgent>();

  useEffect(() => {
    console.log("New Tether Agent with host", props.host);

    const brokerOptions: IClientOptions = {
      ...BROKER_DEFAULTS.browser,
      host: props.host,
    };

    TetherAgent.create("browserDemo", { brokerOptions })
      .then((agent) => {
        setAgent(agent);
        console.info("Tether connect OK");
      })
      .catch((e) => {
        console.error("Error init Tether:", e);
      });
  }, [props.host]);

  return (
    <div>Tether connected? {agent?.getIsConnected() ? "true" : "false"}</div>
  );
};
