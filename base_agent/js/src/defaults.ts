import { Config } from "./types";

const defaults: Config = {
  nodeJS: {
    protocol: "tcp",
    host: "localhost",
    port: 1883,
    path: "",
    username: "tether",
    password: "sp_ceB0ss!",
  },
  browser: {
    protocol: "ws",
    host: "localhost",
    port: 15675,
    path: "/ws",
    username: "tether",
    password: "sp_ceB0ss!",
  },
};

export default defaults;
