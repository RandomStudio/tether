import mqtt, {
  AsyncMqttClient,
  IClientOptions,
  IClientSubscribeOptions,
} from "async-mqtt";
import defaults from "./defaults";
import { v4 as uuidv4 } from "uuid";
import { PlugDefinition } from "./types";
import { Input, Output } from "./Plug";
import logger from "loglevel";
import { LogLevelDesc } from "loglevel";

logger.setLevel("info");
export { logger };

export { Input, Output, IClientOptions };

export class TetherAgent {
  private agentRole: string = null;
  private agentID: string = null;

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
  public static async create(
    agentRole: string,
    agentID: string = "any",
    mqttOptions?: IClientOptions,
    loglevel?: LogLevelDesc
  ): Promise<TetherAgent> {
    const agent = new TetherAgent(agentRole, agentID, loglevel);
    await agent.connect(mqttOptions, false);
    return agent;
  }

  private constructor(
    agentRole: string,
    agentID?: string,
    loglevel?: LogLevelDesc
  ) {
    this.agentRole = agentRole;
    this.agentID = agentID || uuidv4();
    this.client = null;
    if (loglevel) {
      logger.setLevel(loglevel);
    }
    logger.info("Tether Agent instance:", {
      role: this.agentRole,
      id: this.agentID,
    });
  }

  private connect = async (
    overrides?: IClientOptions,
    shouldRetry = true // if MQTT agent retries, it will not throw connection errors!
  ) => {
    const options: IClientOptions = {
      ...defaults.nodeJS,
      ...overrides,
    };
    logger.info("Tether Agent connecting with options", options);

    try {
      this.client = await mqtt.connectAsync(null, options, shouldRetry);
      console.info("Connected OK");
    } catch (error) {
      logger.error("Error connecting to MQTT broker:", {
        error,
        overrides,
        options,
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

  public getRole = () => this.agentRole;
  public getID = () => this.agentID;

  // private listenForIncoming = () => {
  //   this.client.on("message", (topic, payload) => {
  //     const matchingInputPlugs = this.inputs.filter((p) => {
  //       const plugTopic = p.getDefinition().topic;
  //       return topicMatchesPlug(plugTopic, topic);
  //     });
  //     logger.debug("received message:", { topic, payload });
  //     logger.trace(
  //       "available input plugs:",
  //       this.inputs.map((p) => p.getDefinition())
  //     );
  //     logger.debug(
  //       `matched on ${matchingInputPlugs.length}/${this.inputs.length}`,
  //       `plugs`
  //     );
  //     if (matchingInputPlugs.length > 0) {
  //       matchingInputPlugs.forEach((p) => {
  //         p.emitMessage(payload, topic);
  //       });
  //     } else {
  //       logger.warn("message received but cannot match to Input Plug:", {
  //         topic,
  //         payload,
  //       });
  //     }
  //   });
  // };
}
