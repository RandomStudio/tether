import { Config } from "./types";

const defaults: Config = {
  broker: {
    protocol: "tcp",
    host: "tether-io.dev",
    port: 1883,
    path: "",
    username: "tether",
    password: "sp_ceB0ss!",
  },
};

export default defaults;
