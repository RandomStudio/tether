import mqtt, { AsyncMqttClient, IClientOptions } from "async-mqtt";
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
  private agentType: string = null;
  private agentID: string = null;

  private client: AsyncMqttClient | null;

  private inputs: Input[] = [];
  private outputs: Output[] = [];

  public static async create(
    agentType: string,
    overrides?: IClientOptions,
    loglevel?: LogLevelDesc,
    agentID?: string
  ): Promise<TetherAgent> {
    const agent = new TetherAgent(agentType, agentID, loglevel);
    await agent.connect(overrides, false);
    return agent;
  }

  private constructor(
    agentType: string,
    agentID?: string,
    loglevel?: LogLevelDesc
  ) {
    this.agentType = agentType;
    this.agentID = agentID || uuidv4();
    this.client = null;
    if (loglevel) {
      logger.setLevel(loglevel);
    }
    logger.info("Tether Agent instance:", { agentType, agentId: this.agentID });
  }

  private connect = async (
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
    this.outputs.find((p) => p.getDefinition().name === name);

  /**
   * Define a named Output to indicate some data that this agent is expected to produce/send.
   *
   * For convenience, the topic is generated once and used for every message on this Output instance when calling its `publish` function.
   */
  public createOutput = (name: string, overrideTopic?: string) => {
    if (name === undefined) {
      throw Error("No name provided for output");
    }
    if (this.getOutput(name) !== undefined) {
      throw Error(`duplicate plug name "${name}"`);
    }
    if (this.client === null) {
      throw Error("trying to create an Output before client is connected");
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
      throw Error("No name provided for input");
    }
    if (this.getInput(name) !== undefined) {
      throw Error(`duplicate plug name "${name}"`);
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

    setTimeout(() => {
      input
        .subscribe()
        .then(() => {
          logger.info("subscribed OK to", definition.topic);
        })
        .catch((e) => {
          logger.error("failed to subscribe:", e);
        });
    }, this.inputs.length * 100);
    this.inputs.push(input);
    return input;
  };

  private listenForIncoming = () => {
    this.client.on("message", (topic, payload) => {
      const matchingInputPlugs = this.inputs.filter((p) => {
        const plugTopic = p.getDefinition().topic;
        return topicMatchesPlug(plugTopic, topic);
      });
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
          p.emitMessage(payload, topic);
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

export const topicMatchesPlug = (
  plugTopic: string,
  incomingTopic: string
): boolean => {
  if (wasSpecified(plugTopic)) {
    // No wildcards at all in full topic e.g. specified/alsoSpecified/plugName ...
    return plugTopic === incomingTopic;
    // ... Then MATCH only if the defined topic and incoming topic match EXACTLY
  }

  if (wasSpecified(parsePlugName(plugTopic))) {
    if (
      !wasSpecified(parseAgentType(plugTopic)) &&
      !wasSpecified(parseAgentIdOrGroup(plugTopic))
      // if ONLY the Plug Name was specified (which is the default), then MATCH
      // anything that matches the Plug Name, regardless of the rest
    ) {
      return parsePlugName(plugTopic) === parsePlugName(incomingTopic);
    }

    // If either the AgentType or ID/Group was specified, check these as well...

    // if AgentType specified, see if this matches, otherwise pass all AgentTypes as matches
    // e.g. specified/+/plugName
    const agentTypeMatches = wasSpecified(parseAgentType(plugTopic))
      ? parseAgentType(plugTopic) === parseAgentType(incomingTopic)
      : true;

    // if Agent ID or Group specified, see if this matches, otherwise pass all AgentIdOrGroup as matches
    // e.g. +/specified/plugName
    const agentIdOrGroupMatches = wasSpecified(parseAgentIdOrGroup(plugTopic))
      ? parseAgentIdOrGroup(plugTopic) === parseAgentIdOrGroup(incomingTopic)
      : true;

    return agentTypeMatches && agentIdOrGroupMatches;
  } else {
    // something/something/+ is not allowed for Plugs
    logger.error("No PlugName was specified for this Plug:", plugTopic);
    return false;
  }
};

const wasSpecified = (topicOrPart: string) => !topicOrPart.includes("+");

// const topicMatchesNameOnly = (topic: string) => hasWildcards(parseAgentID(topic)) && hasWildcards(parseAgentType(topic));
// const topicMatchesIdOnly = (topic: string) => hasWildcards(parsePlugName(topic)) && hasWildcards(parseAgentType(topic));
// const topicMatchesGroupOnly = (topic: string) => hasWildcards(parsePlugName(topic)) && hasWildcards(parsePlugName(topic));

export const parsePlugName = (topic: string) => topic.split(`/`)[2];
export const parseAgentIdOrGroup = (topic: string) => topic.split(`/`)[1];
export const parseAgentType = (topic: string) => topic.split(`/`)[0];
