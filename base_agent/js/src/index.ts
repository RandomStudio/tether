import mqtt, { AsyncMqttClient, IClientOptions } from "async-mqtt";
import defaults from "./defaults";
import { v4 as uuidv4 } from "uuid";
import { PlugDefinition } from "./types";
import { Input, Output } from "./Plug";

const { getLogger } = require("log4js");

export const logger = getLogger("tetherAgentJS");
logger.level = "info";

export const connectTetherAgent = async (
  agentType: string,
  agentID?: string,
  overrides?: IClientOptions,
  loglevel?: string
): Promise<TetherAgent> => {
  const agent = new TetherAgent(agentType, agentID, loglevel);
  await agent.connect(overrides, false);
  return agent;
};
class TetherAgent {
  private agentType: string = null;
  private agentID: string = null;

  private client: AsyncMqttClient | null;

  private inputs: Input[] = [];
  private outputs: Output[] = [];

  constructor(agentType: string, agentID?: string, loglevel?: string) {
    this.agentType = agentType;
    this.agentID = agentID || uuidv4();
    this.client = null;
    if (loglevel) {
      logger.level = loglevel;
    }
    logger.info("Tether Agent instance:", { agentType, agentId: this.agentID });
  }

  public connect = async (
    overrides?: IClientOptions,
    shouldRetry = true // if MQTT agent retries, it will not throw connection errors!
  ) => {
    const options: IClientOptions = {
      ...defaults.broker,
      ...overrides,
    };
    logger.info("Tether Agent connecting with options", options);

    try {
      this.client = await mqtt.connectAsync(null, options, shouldRetry);
      console.info("Connected OK");
      this.listenForIncoming();
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

  public getInput = (name: string) =>
    this.inputs.find((p) => p.getDefinition().name === name);

  public getOutput = (name: string) =>
    this.inputs.find((p) => p.getDefinition().name === name);

  /**
   * Define a named Output to indicate some data that this agent is expected to produce/send.
   *
   * For convenience, the topic is generated once and used for every message on this Output instance when calling its `publish` function.
   */
  public createOutput = (name: string, overrideTopic?: string) => {
    if (name === undefined) {
      throw Error("No name provided for output");
    }
    if (this.client === null) {
      logger.warn(
        "Created output before client connected. This is allowed but you will be unable to publish messages until connected."
      );
    }
    const definition: PlugDefinition = {
      name,
      topic: overrideTopic || `${this.agentType}/${this.agentID}/${name}`,
    };

    const output = new Output(this.client, definition);
    this.outputs.push(output);

    return output;
  };

  /**
   * Define a named Input to indicate some data this agent is expected to consume/receive.
   *
   * For convenience, the topic is assumed to end in the given `name`, e.g. an Input named "someTopic" will match messages on topics `foo/bar/someTopic` as well as `something/else/someTopic`.
   */
  public createInput = (name: string, overrideTopic?: string) => {
    if (name === undefined) {
      throw Error("No name provided for output");
    }

    if (this.client === null) {
      throw Error("trying to create an Input before client is connected");
    }

    // Create a new Input
    const definition: PlugDefinition = {
      name,
      topic: overrideTopic || `+/+/${name}`,
    };
    const input = new Input(this.client, definition);

    input
      .subscribe()
      .then(() => {
        logger.debug("subscribed ok");
      })
      .catch((e) => {
        logger.error("failed to subscribe:", e);
      });

    this.inputs.push(input);

    return input;
  };

  private listenForIncoming = () => {
    this.client.on("message", (topic, payload) => {
      const matchingInputPlugs = this.inputs.filter((p) =>
        // If the Plug was defined with a wildcard anywhere, match
        // on name, i.e. last part of 3-part topic agentType/agentGroup/name
        // Otherwise, match on topic exactly
        topicHasWildcards(p.getDefinition().topic)
          ? getTopicPlugName(p.getDefinition().topic) ===
            getTopicPlugName(topic)
          : p.getDefinition().topic === topic
      );
      logger.debug("received message:", { topic, payload });
      logger.trace(
        "available input plugs:",
        this.inputs.map((p) => p.getDefinition())
      );
      logger.debug(
        `matched on ${matchingInputPlugs.length}/${this.inputs.length}`,
        `plugs`
      );
      if (matchingInputPlugs.length > 0) {
        matchingInputPlugs.forEach((p) => {
          p.emit("message", payload, topic);
        });
      } else {
        logger.warn("message received but cannot match to Input Plug:", {
          topic,
          payload,
        });
      }
    });
  };
}

const topicHasWildcards = (topic: string) => topic.includes("+");

const getTopicPlugName = (topic: string) => topic.split(`/`)[2];
