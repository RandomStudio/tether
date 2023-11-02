import { IClientOptions } from "async-mqtt";

export interface PlugDefinition {
  name: string;
  topic: string;
}

export type MessageCallback = (payload: Buffer, topic: string) => void;

export interface TetherConfig {
  role: string;
  id: string;
  brokerOptions: IClientOptions;
  autoConnect: boolean;
}

export interface TetherOptions {
  role: string;
  id?: string;
  brokerOptions?: IClientOptions;
  autoConnect: boolean;
  loglevel?: string;
}
