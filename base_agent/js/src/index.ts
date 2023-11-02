import mqtt, {
  AsyncMqttClient,
  IClientOptions,
  IClientSubscribeOptions,
} from "async-mqtt";
import defaults, { BROKER_DEFAULTS } from "./defaults";
import { Input, Output } from "./Plug";
import logger from "loglevel";
import { LogLevelDesc } from "loglevel";
import { TetherConfig, TetherOptions } from "./types";

logger.setLevel("info");
export { logger, BROKER_DEFAULTS as DEFAULTS };

export { Input, Output, IClientOptions };

export class TetherAgent {
  private config: TetherConfig;
  private client: AsyncMqttClient | null;

  /**
   * Create a new Tether Agent, and connect automatically. This is an async function, so it will return the
   * agent via the Promise only once the connection has been made successfully.
   *
   * @param agentRole The Role of this Agent in the system. Describes what this type of Agent does.
   * @param agentID An optional identifier for the Agent. Could be a unique ID for this instance or something shared for a group of agents. Defaults to "any".
   * @param mqttOptions Connection details (host, port, etc.) for the MQTT Broker. Leave this out to use defaults.
   * @param loglevel Make the Tether library more verbose by setting "debug", for example.
   * @returns
   */
  public static async create(options: TetherOptions): Promise<TetherAgent> {
    const config: TetherConfig = {
      role: options.role || defaults.role,
      id: options.id || defaults.id,
      brokerOptions: options.brokerOptions || defaults.brokerOptions,
      autoConnect: options.autoConnect || defaults.autoConnect,
    };
    const agent = new TetherAgent(
      config,
      (options.loglevel || "info") as LogLevelDesc
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
