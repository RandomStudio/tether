import mqtt, { AsyncMqttClient, IClientOptions } from "async-mqtt";
import { EventEmitter } from "events";
import defaults from "./defaults";
import { v4 as uuidv4 } from "uuid";
import { PlugDefinition } from "./types";

const { getLogger } = require("log4js");

const logger = getLogger("tetherAgentJS");
logger.level = "info";
class Plug extends EventEmitter {
  protected definition: PlugDefinition;
  protected client: AsyncMqttClient | null;

  constructor(client: AsyncMqttClient, definition: PlugDefinition) {
    super();
    this.client = client;
    this.definition = definition;
  }

  public getDefinition = () => this.definition;
}
export class Input extends Plug {
  subscribe = async () => {
    if (this.client === null) {
      logger.warn(
        "subscribing to topic before client is connected; this is allowed but you won't receive any messages until connected"
      );
    }
    await this.client.subscribe(this.definition.topic);
    logger.debug("subscribed to topic", this.definition.topic);
  };
}

export class Output extends Plug {
  publish = async (content: Buffer | Uint8Array) => {
    if (this.client === null) {
      logger.error(
        "trying to send without connection; not possible until connected"
      );
    } else {
      if (content instanceof Uint8Array) {
        this.client.publish(this.definition.topic, Buffer.from(content));
      } else {
        this.client.publish(this.definition.topic, content);
      }
    }
  };
}

export class TetherAgent {
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
    const definition: PlugDefinition = {
      name,
      topic: overrideTopic || `${this.agentType}/${this.agentID}/${name}`,
      flowDirection: "out",
    };

    const output = new Output(this.client, definition);
    this.outputs.push(output);

    return output;
  };

  /**
   * Define a named Input to indicate some data this agent is expected to consume/receive.
   *
   * For convenience, the topic is assumed to end in the given `name`, e.g. an Input named "someTopic" will match messages on topics `foo/bar/someTopic` as well as `something/else/someTopic`.
   *
   * Returns an Output instance which is an EventEmitter. Events named "message" with contents (topic, message) will be emitted on this instance, but _only_ if they match the Output name.
   */
  public createInput = (name: string, overrideTopic?: string) => {
    if (name === undefined) {
      throw Error("No name provided for output");
    }

    // Create a new Input
    const definition: PlugDefinition = {
      name,
      topic: overrideTopic || `+/+/${name}`,
      flowDirection: "in",
    };
    const input = new Input(this.client, definition);

    input.subscribe();

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
      logger.debug("matched on", matchingInputPlugs.length, "plugs");
      if (matchingInputPlugs.length > 0) {
        matchingInputPlugs.forEach((p) => {
          p.emit("message", topic, payload);
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

const topicHasPlugName = (topic: string, plugName: string) =>
  getTopicPlugName(topic) === plugName;

// export default TetherAgent;
