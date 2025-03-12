import { IClientOptions } from "async-mqtt";
import { BROWSER, NODEJS } from "./defaults";
import { InputPlug, OutputPlug } from "./Plug";
import logger from "loglevel";
import { encode, decode } from "@msgpack/msgpack";
import { TetherAgent } from "./Agent";
export {
  parsePlugName,
  parseAgentIdOrGroup,
  parseAgentRole as parseAgentType,
} from "./Plug";

logger.setLevel("info");
export { logger, BROWSER, NODEJS, encode, decode };
export { TetherAgent, InputPlug, OutputPlug, IClientOptions };
