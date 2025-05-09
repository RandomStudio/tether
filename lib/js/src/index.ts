import { IClientOptions } from "async-mqtt";
import { BROWSER, NODEJS } from "./defaults";
import { getLogger } from "log4js";
import { encode, decode } from "@msgpack/msgpack";
import { TetherAgent } from "./Agent";
import { ChannelReceiver } from "./Channel/ChannelReceiver";
import { ChannelSender } from "./Channel/ChannelSender";
export {
  parseChannelName,
  parseAgentIdOrGroup,
  parseAgentRole,
} from "./Channel";

const logger = getLogger("TetherAgent");

export { logger, BROWSER, NODEJS, encode, decode };
export { TetherAgent, ChannelReceiver, ChannelSender, IClientOptions };
