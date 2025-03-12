import { IClientOptions } from "async-mqtt";
import { TetherAgentConfig } from "./types";

export const NODEJS: IClientOptions = {
  protocol: "tcp",
  host: "localhost",
  port: 1883,
  path: "",
  username: "tether",
  password: "sp_ceB0ss!",
};

export const BROWSER: IClientOptions = {
  protocol: "ws",
  host: "localhost",
  port: 15675,
  path: "/ws",
  username: "tether",
  password: "sp_ceB0ss!",
};

const defaults: TetherAgentConfig = {
  role: "unknownTetherAgent",
  brokerOptions: NODEJS,
  autoConnect: true,
};

export default defaults;
