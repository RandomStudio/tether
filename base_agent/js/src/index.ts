import mqtt, { AsyncMqttClient, IClientOptions } from "async-mqtt";
import defaults, { BROKER_DEFAULTS } from "./defaults";
import { InputPlug, OutputPlug } from "./Plug";
import logger from "loglevel";
import { LogLevelDesc } from "loglevel";
import { TetherConfig, TetherOptions } from "./types";
import { encode, decode } from "@msgpack/msgpack";
export { parsePlugName, parseAgentIdOrGroup, parseAgentType } from "./Plug";

logger.setLevel("info");
export { logger, BROKER_DEFAULTS, encode, decode };

export { InputPlug, OutputPlug, IClientOptions };

enum State {
  INITIALISED = "INITIALISED",
  CONNECTING = "CONNECTING",
  ERRORED = "ERRORED",
  CONNECTED = "CONNECTED",
}

export class TetherAgent {
  private state: State;
  private config: TetherConfig;
  private client: AsyncMqttClient | null;

  /**
   * Create a new Tether Agent, and connect automatically unless options.autoConnect is.set to `false`.
   * This is an async function, so it will return the agent via the Promise only once the connection
   * has been made successfully - or immediately if you turn autoConnect off.
   *
   * @param role The Role of this Agent in the system. Describes what this type of Agent does.
   * @param options Optional object which in turn has optional fields. Use this to override defaults
   * as necessary. For example, `{ brokerOptions.BROKER_DEFAULTS.browser }` will switch MQTT Client
   * options to use sensible defaults for WebSocket connections, instead of the default
   * TCP connection which is more suitable for NodeJS applications.
   * @returns A new TetherAgent instance
   */
  public static async create(
    role: string,
    options?: TetherOptions
  ): Promise<TetherAgent> {
    if (options?.loglevel) {
      logger.setLevel(options.loglevel as LogLevelDesc);
    }
    if (
      options &&
      options.brokerOptions &&
      options.brokerOptions.host !== undefined &&
      options.brokerOptions.hostname === undefined
    ) {
      logger.warn(
        "hostname should be same as host; we set this for you to avoid confusion"
      );
      options.brokerOptions.hostname = options.brokerOptions.host;
    }
    const config: TetherConfig = {
      role,
      id: options?.id || defaults.id,
      brokerOptions: options?.brokerOptions || defaults.brokerOptions,
      autoConnect: options?.autoConnect || defaults.autoConnect,
    };
    const agent = new TetherAgent(
      config,
      (options?.loglevel || "info") as LogLevelDesc
    );
    if (config.autoConnect === true) {
      try {
        await agent.connect();
      } catch (e) {
        logger.error("Error on auto-connect:", e);
        // agent.state = State.ERRORED;
      }
    } else {
      logger.warn(
        "Tether Agent was initialised without auto-connect. You will need to call .connect() yourself."
      );
    }
    return agent;
  }

  private constructor(config: TetherConfig, loglevel?: LogLevelDesc) {
    this.config = config;
    this.client = null;
    this.state = State.INITIALISED;
    if (loglevel) {
      logger.setLevel(loglevel);
    }
    logger.info("Tether Agent instance:", { role: config.role, id: config.id });
  }

  public connect = async () => {
    logger.info("Tether Agent connecting with options", {
      ...this.config.brokerOptions,
    });
    this.state = State.CONNECTING;

    try {
      const client = await mqtt.connectAsync(
        null,
        {
          ...this.config.brokerOptions,
          clientId: `tether-${this.config.role}`,
        },
        false
      );
      console.info("Connected OK");
      this.client = client;
      this.state = State.CONNECTED;
    } catch (error) {
      logger.error("Error connecting to MQTT broker:", {
        error,
        brokerOptions: this.config.brokerOptions,
      });
      this.client = null;
      this.state = State.ERRORED;
      throw error;
    }
  };

  public disconnect = async () => {
    if (this.client) {
      await this.client.end();
      this.client = null;
      logger.debug("MQTT client closed normally");
      this.state = State.INITIALISED;
    } else {
      logger.warn("MQTT client not available on disconnect request");
    }
  };

  /**
   * End users can get the underlying client if they like. This allows you to bypass
   * the "plugs" altogether and subscribe or publish on topics directly.
   *
   * @returns The AsyncMqttClient client instance, or null if not (yet) connected
   */
  public getClient = () => this.client;

  public getState = () => this.state;

  public getIsConnected = () =>
    this.client !== null &&
    this.client.connected &&
    this.state === State.CONNECTED;

  public getConfig = () => this.config;
}
