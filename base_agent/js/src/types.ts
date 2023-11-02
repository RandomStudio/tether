import { IClientOptions } from "async-mqtt";

export interface Config {
  nodeJS: IClientOptions;
  browser: IClientOptions;
}

export interface PlugDefinition {
  name: string;
  topic: string;
}

export type MessageCallback = (payload: Buffer, topic: string) => void;
