import { IClientOptions } from "async-mqtt";

export interface PlugDefinition {
  name: string;
  id?: string;
  topic: string;
}

export type MessageCallback = (payload: Buffer, topic: string) => void;

export interface TetherAgentConfig {
  role: string;
  id?: string;
  brokerOptions: IClientOptions;
  autoConnect: boolean;
}

export interface TetherOptions {
  id?: string;
  brokerOptions?: IClientOptions;
  autoConnect?: boolean;
  loglevel?: string;
}
