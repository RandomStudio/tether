import mqtt, { AsyncMqttClient } from "async-mqtt";
import { TetherAgentConfig, TetherOptions } from "./types";
import { ChannelReceiver, ReceiverOptions } from "./Channel/ChannelReceiver";
import { logger } from "./";
import defaults from "./defaults";
import { ChannelSender, SenderOptions } from "./Channel/ChannelSender";

enum State {
  INITIALISED = "INITIALISED",
  CONNECTING = "CONNECTING",
  ERRORED = "ERRORED",
  CONNECTED = "CONNECTED",
}

export class TetherAgent {
  private state: State;
  private config: TetherAgentConfig;
  private client: AsyncMqttClient | null;

  private mySenders: ChannelSender<any>[];
  private myReceivers: ChannelReceiver<any>[];

  /**
   * Create a new Tether Agent, and connect automatically unless options.autoConnect is.set to `false`.
   * This is an async function, so it will return the agent via the Promise only once the connection
   * has been made successfully - or immediately if you turn autoConnect off.
   *
   * @param role The Role of this Agent in the system. Describes what this type of Agent does.
   * @param options Optional object which in turn has optional fields. Use this to override defaults
   * as necessary. For example, `{ brokerOptions: BROWSER }` will switch MQTT Client
   * options to use sensible defaults for WebSocket connections, instead of the default
   * TCP connection which is more suitable for NodeJS applications.
   * @returns A new TetherAgent instance
   */
  public static async create(
    role: string,
    options?: TetherOptions
  ): Promise<TetherAgent> {
    if (options?.loglevel) {
      logger.level = options.loglevel;
    }
    if (
      options &&
      options.brokerOptions &&
      options.brokerOptions.host !== undefined &&
      options.brokerOptions.hostname === undefined
    ) {
      logger.debug(
        "hostname should be same as host; we set this for you to avoid confusion"
      );
      options.brokerOptions.hostname = options.brokerOptions.host;
    }
    const config: TetherAgentConfig = {
      role,
      id: options?.id,
      brokerOptions: options?.brokerOptions ?? defaults.brokerOptions,
      autoConnect: options?.autoConnect ?? defaults.autoConnect,
    };
    const agent = new TetherAgent(config, options?.loglevel || "warn");
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

  private constructor(config: TetherAgentConfig, loglevel?: string) {
    this.config = config;
    this.client = null;
    this.state = State.INITIALISED;
    this.myReceivers = [];
    this.mySenders = [];
    if (loglevel) {
      logger.level = loglevel;
    }
    logger.info(`Tether Agent instance with role "${config.role}"`);
    logger.debug("TetherAgent log level:", logger.level);
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
      logger.info("Connected OK");
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
    logger.warn("Tether Agent explicit disconnect requested");
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
   * the "Channels" altogether and subscribe or publish on topics directly.
   *
   * @returns The AsyncMqttClient client instance, or null if not (yet) connected
   */
  public getClient = () => this.client;

  public getState = () => this.state;

  public getIsConnected = () => {
    const isConnected =
      this.client !== null &&
      this.client.connected === true &&
      this.state === State.CONNECTED;

    if (!isConnected) {
      logger.error(
        "Not connected! Client Null?",
        typeof this.client,
        "; client.connected?",
        this.client?.connected,
        "; state?",
        this.state
      );
    }

    return isConnected;
  };

  public getConfig = () => this.config;

  public async createReceiver<T>(
    name: string,
    options?: ReceiverOptions,
    ignoreExisting?: boolean
  ): Promise<ChannelReceiver<T>> {
    const existing = this.myReceivers.find(
      (c) => c.getDefinition().name === name
    );
    logger.debug({
      name,
      existing: existing?.getDefinition(),
      options,
      ignoreExisting,
    });
    if (existing && ignoreExisting !== true) {
      logger.warn(`Channel "${name}" already exists; will re-use`);
      return existing;
    } else {
      const receiver = (await ChannelReceiver.create(
        this,
        name
      )) as ChannelReceiver<T>;
      this.myReceivers.push(receiver);
      return receiver;
    }
  }

  public createSender<T>(
    name: string,
    options?: SenderOptions,
    ignoreExisting?: boolean
  ): ChannelSender<T> {
    const existing = this.mySenders.find(
      (c) => c.getDefinition().name === name
    );
    // logger.debug({
    //   name,
    //   existing: existing?.getDefinition(),
    //   options,
    //   ignoreExisting,
    // });
    if (existing && ignoreExisting !== true) {
      logger.warn(`Channel "${name}" already exists; will re-use`);
      return existing;
    } else {
      const sender = new ChannelSender(this, name);
      this.mySenders.push(sender);
      return sender;
    }
  }
}
