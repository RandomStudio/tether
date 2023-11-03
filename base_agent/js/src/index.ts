import mqtt, { AsyncMqttClient, IClientOptions } from "async-mqtt";
import defaults, { BROKER_DEFAULTS } from "./defaults";
import { InputPlug, OutputPlug } from "./Plug";
import logger from "loglevel";
import { LogLevelDesc } from "loglevel";
import { TetherConfig, TetherOptions } from "./types";
import { encode, decode } from "@msgpack/msgpack";

logger.setLevel("info");
export { logger, BROKER_DEFAULTS, encode, decode };

export { InputPlug, OutputPlug, IClientOptions };

export class TetherAgent {
  private config: TetherConfig;
  private client: AsyncMqttClient | null;

  /**
   * Create a new Tether Agent, and connect automatically unless . This is an async
   * function, so it will return the agent via the Promise only once the connection
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
    if (
      options &&
      options.brokerOptions &&
      options.brokerOptions.host !== undefined &&
      options.brokerOptions.hostname === undefined
    ) {
      logger.warn("hostname must be same as host!");
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
      await agent.connect();
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
    if (loglevel) {
      logger.setLevel(loglevel);
    }
    logger.info("Tether Agent instance:", { role: config.role, id: config.id });
  }

  public connect = async () => {
    logger.info("Tether Agent connecting with options", {
      ...this.config.brokerOptions,
    });

    try {
      this.client = await mqtt.connectAsync(
        null,
        this.config.brokerOptions,
        false
      );
      console.info("Connected OK");
    } catch (error) {
      logger.error("Error connecting to MQTT broker:", {
        error,
        brokerOptions: this.config.brokerOptions,
      });
      throw error;
    }
  };

  public disconnect = async () => {
    if (this.client) {
      await this.client.end();
      logger.debug("MQTT client closed normally");
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

  public getIsConnected = () => this.client !== null;

  public getConfig = () => this.config;
}
