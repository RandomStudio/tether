import { IClientOptions } from "async-mqtt";
import { TetherConfig } from "./types";

const NODEJS: IClientOptions = {
  protocol: "tcp",
  host: "localhost",
  port: 1883,
  path: "",
  username: "tether",
  password: "sp_ceB0ss!",
};

const BROWSER: IClientOptions = {
  protocol: "ws",
  host: "localhost",
  port: 15675,
  path: "/ws",
  username: "tether",
  password: "sp_ceB0ss!",
};

export const BROKER_DEFAULTS = {
  nodeJS: NODEJS,
  browser: BROWSER,
};

const defaults: TetherConfig = {
  role: "unknown",
  id: "any",
  brokerOptions: NODEJS,
  autoConnect: true,
};

export default defaults;
