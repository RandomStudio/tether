import { IClientOptions } from "async-mqtt";

export interface Config {
  broker: IClientOptions;
}

export interface PlugDefinition {
  name: string;
  topic: string;
  flowDirection: "in" | "out";
}
